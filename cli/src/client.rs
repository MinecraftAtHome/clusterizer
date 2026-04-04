use std::{
    collections::{HashMap, VecDeque},
    env,
    ffi::OsString,
    fs,
    io::{Cursor, ErrorKind},
    iter::{self, Empty},
    path::Path,
    process::{Output, Stdio},
    sync::Arc,
    time::Duration,
};

use clusterizer_api::{client::ApiClient, result::ApiError};
use clusterizer_client::result::ClientResult;
use clusterizer_common::{
    errors::SubmitResultError,
    records::{
        File, FileFilter, Platform, PlatformFilter, Project, ProjectFilter, ProjectVersion,
        ProjectVersionFilter, Task,
    },
    requests::{FetchTasksRequest, SubmitResultRequest},
    types::Id,
};
use tokio::{io::AsyncWriteExt, process::Command, task::JoinSet, time};
use tracing::{debug, info, warn};
use zip::ZipArchive;

use crate::args::RunArgs;

struct ClusterizerClient {
    client: ApiClient,
    args: RunArgs,
    platform_ids: Vec<Id<Platform>>,
}

struct TaskInfo {
    task: Task,
    project: Project,
    project_version: ProjectVersion,
    file: File,
}

enum Return {
    FetchTasks(Vec<TaskInfo>),
    ExecuteTask(Id<Task>, Output),
    SubmitResult,
}

impl ClusterizerClient {
    async fn run(self: Arc<Self>) -> ClientResult<()> {
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
            let projects: HashMap<_, _> = self
                .client
                .get_all::<Project>(&ProjectFilter::default().disabled(false))
                .await?
                .into_iter()
                .map(|project| (project.id, project))
                .collect();

            let project_versions: HashMap<_, _> = self
                .client
                .get_all::<ProjectVersion>(&ProjectVersionFilter::default().disabled(false))
                .await?
                .into_iter()
                .filter(|project_version| self.platform_ids.contains(&project_version.platform_id))
                .map(|project_version| (project_version.id, project_version))
                .collect();

            let files: HashMap<_, _> = self
                .client
                .get_all::<File>(&FileFilter::default())
                .await?
                .into_iter()
                .filter(|file| {
                    project_versions
                        .iter()
                        .any(|(_, project_version)| project_version.file_id == file.id)
                })
                .filter_map(|file| {
                    project_versions
                        .values()
                        .find(|project_version| project_version.file_id == file.id)
                        .map(|project_version| (project_version.project_id, file))
                })
                .collect();

            let tasks: Vec<_> = self
                .client
                .fetch_tasks(&FetchTasksRequest {
                    project_ids: projects.keys().copied().collect(),
                    limit: self.args.threads,
                })
                .await?
                .into_iter()
                .filter_map(|task| {
                    let project = projects.get(&task.project_id)?;
                    let file = files.get(&task.project_id)?;
                    let project_version = project_versions
                        .iter()
                        .find(|(_, project_version)| project_version.file_id == file.id)?;
                    let info = files.get(&task.project_id).map(|file| TaskInfo {
                        task,
                        file: file.clone(),
                        project: project.clone(),
                        project_version: project_version.1.clone(),
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

        for TaskInfo { file, .. } in &tasks {
            download_archive(
                &file.url,
                self.args
                    .cache_dir
                    .join("bin")
                    .join(format!("{:x}", file))
                    .as_path(),
                &self.args.cache_dir,
            )
            .await?;
        }

        Ok(Return::FetchTasks(tasks))
    }

    async fn execute_task(
        self: Arc<Self>,
        TaskInfo {
            task,
            project_version,
            project,
            file,
        }: TaskInfo,
    ) -> ClientResult<Return> {
        let slot_dir = tempfile::tempdir()?;

        info!("Task id: {}, stdin: {}", task.id, task.stdin);
        info!(
            "Project id: {}, Project name: {}",
            task.project_id, project.name
        );
        debug!("Platform id: {}", project_version.platform_id);
        debug!("Slot dir: {}", slot_dir.path().display());

        let program_dir = self.args.cache_dir.join("bin").join(format!("{:x}", file));

        let program = program_dir
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

        match self.client.submit_result(task_id, &request).await {
            Err(ApiError::Specific(SubmitResultError::AssignmentExpired)) => {}
            result => result?,
        };

        Ok(Return::SubmitResult)
    }
}

pub async fn run(client: ApiClient, args: RunArgs) -> ClientResult<()> {
    fs::create_dir_all(args.binaries_dir())?;

    let mut platform_ids = Vec::new();
    let mut platform_names = Vec::new();

    for platform in client
        .get_all::<Platform>(&PlatformFilter::default())
        .await?
    {
        let file = client.get_one(platform.file_id).await?;

        debug!(
            "Platform id: {}, tester archive url: {}",
            platform.id, file.url
        );

        let platform_tester_dir = args.binaries_dir().join(format!("{:x}", file));

        download_archive(&file.url, &platform_tester_dir, &args.cache_dir).await?;

        let slot_dir = tempfile::tempdir()?;

        debug!("Slot dir: {}", slot_dir.path().display());

        let program = match platform_tester_dir
            .join(format!("main{}", env::consts::EXE_SUFFIX))
            .canonicalize()
        {
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            result => result,
        }?;

        let args: Empty<OsString> = iter::empty();

        let status = Command::new(program)
            .args(args)
            .current_dir(&slot_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        if status.success() {
            platform_ids.push(platform.id);
            platform_names.push(platform.name);
        }
    }

    info!("Supported platforms: {}", platform_names.join(", "));

    Arc::new(ClusterizerClient {
        client,
        args,
        platform_ids,
    })
    .run()
    .await
}

async fn download_archive(url: &str, dir: &Path, cache_dir: &Path) -> ClientResult<()> {
    if dir.exists() {
        debug!("Archive {} was cached.", dir.display());
    } else {
        debug!("Archive {} is not cached.", dir.display());

        let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
        let extract_dir = tempfile::tempdir_in(cache_dir)?;

        ZipArchive::new(Cursor::new(bytes))?.extract(&extract_dir)?;
        fs::rename(&extract_dir, dir)?;
    }

    Ok(())
}
