use std::{
    collections::{HashMap, VecDeque},
    env,
    ffi::OsString,
    fs,
    io::Cursor,
    iter::{self, Empty},
    process::{Output, Stdio},
    sync::Arc,
    time::Duration,
};

use clusterizer_api::client::ApiClient;
use clusterizer_common::{
    records::{Project, ProjectFilter, ProjectVersion, ProjectVersionFilter, Task},
    requests::{FetchTasksRequest, SubmitResultRequest},
    types::Id,
};
use log::{debug, info, warn};
use tokio::{io::AsyncWriteExt, process::Command, task::JoinSet, time};
use zip::ZipArchive;

use crate::{args::RunArgs, result::ClientResult};

pub struct ClusterizerClient {
    client: ApiClient,
    args: RunArgs,
}

struct TaskInfo {
    task: Task,
    project: Project,
    project_version: ProjectVersion,
}

enum Return {
    FetchTasks(Vec<TaskInfo>),
    ExecuteTask(Id<Task>, Output),
    SubmitResult,
}

impl ClusterizerClient {
    pub fn new(client: ApiClient, args: RunArgs) -> ClusterizerClient {
        ClusterizerClient { client, args }
    }

    pub async fn run(self: Arc<Self>) -> ClientResult<()> {
        fs::create_dir_all(&self.args.cache_dir)?;

        let mut set = JoinSet::new();
        let mut tasks = VecDeque::new();
        let mut fetching_tasks = true;
        let mut used_threads = 0;

        set.spawn(Arc::clone(&self).fetch_tasks());

        while let Some(ret) = set.join_next().await {
            match ret?? {
                Return::FetchTasks(new_tasks) => {
                    fetching_tasks = false;
                    tasks.extend(new_tasks);
                }
                Return::ExecuteTask(task_id, output) => {
                    used_threads -= 1;
                    set.spawn(Arc::clone(&self).submit_result(task_id, output));
                }
                Return::SubmitResult => {}
            }

            let mut out_of_tasks = false;

            while used_threads < self.args.threads {
                if let Some(task) = tasks.pop_front() {
                    used_threads += 1;
                    set.spawn(Arc::clone(&self).execute_task(task));
                } else {
                    out_of_tasks = true;

                    break;
                }
            }

            if !fetching_tasks && (out_of_tasks || tasks.len() < self.args.queue) {
                fetching_tasks = true;
                set.spawn(Arc::clone(&self).fetch_tasks());
            }
        }

        Ok(())
    }

    async fn fetch_tasks(self: Arc<Self>) -> ClientResult<Return> {
        let tasks = loop {
            let mut projects: HashMap<_, _> = self
                .client
                .get_all::<Project>(&ProjectFilter::default())
                .await?
                .into_iter()
                .map(|project| (project.id, project))
                .collect();

            let projects: HashMap<_, _> = self
                .client
                .get_all::<ProjectVersion>(
                    &ProjectVersionFilter::default()
                        .platform_id(self.args.platform_id)
                        .disabled(false),
                )
                .await?
                .into_iter()
                .filter_map(|project_version| {
                    projects
                        .remove(&project_version.project_id)
                        .map(|project| (project.id, (project, project_version)))
                })
                .collect();

            let tasks: Vec<_> = self
                .client
                .fetch_tasks(&FetchTasksRequest {
                    project_ids: projects.keys().copied().collect(),
                })
                .await?
                .into_iter()
                .filter_map(|task| {
                    let info = projects
                        .get(&task.project_id)
                        .map(|(project, project_version)| TaskInfo {
                            task,
                            project: project.clone(),
                            project_version: project_version.clone(),
                        });

                    if info.is_none() {
                        warn!("Unwanted task received from server.");
                    }

                    info
                })
                .collect();

            if !tasks.is_empty() {
                break tasks;
            }

            info!("No tasks found. Sleeping before attempting again.");
            time::sleep(Duration::from_millis(15000)).await;
        };

        for TaskInfo {
            project_version, ..
        } in &tasks
        {
            let project_version_dir = self.args.cache_dir.join(project_version.id.to_string());

            if project_version_dir.exists() {
                debug!("Archive {} was cached.", project_version_dir.display());
            } else {
                debug!("Archive {} is not cached.", project_version_dir.display());

                let bytes = reqwest::get(&project_version.archive_url)
                    .await?
                    .error_for_status()?
                    .bytes()
                    .await?;

                let extract_dir = tempfile::tempdir_in(&self.args.cache_dir)?;

                ZipArchive::new(Cursor::new(bytes))?.extract(&extract_dir)?;
                fs::rename(&extract_dir, project_version_dir)?;
            }
        }

        Ok(Return::FetchTasks(tasks))
    }

    async fn execute_task(
        self: Arc<Self>,
        TaskInfo {
            task,
            project,
            project_version,
        }: TaskInfo,
    ) -> ClientResult<Return> {
        let slot_dir = tempfile::tempdir()?;

        info!("Task id: {}, stdin: {}", task.id, task.stdin);
        info!("Project id: {}, name: {}", project.id, project.name);
        debug!(
            "Project version id: {}, archive url: {}",
            project_version.id, project_version.archive_url
        );
        debug!("Slot dir: {}", slot_dir.path().display());

        let program = self
            .args
            .cache_dir
            .join(project_version.id.to_string())
            .join(format!("main{}", env::consts::EXE_SUFFIX))
            .canonicalize()?;

        let args: Empty<OsString> = iter::empty();

        let mut child = Command::new(program)
            .args(args)
            .current_dir(&slot_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();

        tokio::spawn(async move {
            stdin.write_all(task.stdin.as_bytes()).await.unwrap();
        });

        let output = child.wait_with_output().await?;

        Ok(Return::ExecuteTask(task.id, output))
    }

    async fn submit_result(
        self: Arc<Self>,
        task_id: Id<Task>,
        output: Output,
    ) -> ClientResult<Return> {
        let request = SubmitResultRequest {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code(),
        };

        self.client.submit_result(task_id, &request).await?;

        Ok(Return::SubmitResult)
    }
}
