use std::{
    collections::VecDeque,
    process::{Output, Stdio},
    sync::Arc,
    time::Duration,
};

use clusterizer_api::{client::ApiClient, result::ApiError};
use clusterizer_client::{TaskInfo, result::ClientResult, supported_platforms::SupportedPlatforms};
use clusterizer_common::{
    errors::SubmitResultError, records::Task, requests::SubmitResultRequest, types::Id,
};
use tokio::{io::AsyncWriteExt, task::JoinSet, time};
use tracing::{debug, info};

use crate::args::RunArgs;

struct ClusterizerClient {
    client: ApiClient,
    args: RunArgs,
    supported_platforms: SupportedPlatforms,
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
        loop {
            let tasks = clusterizer_client::fetch_tasks(
                &self.args.cache_dir,
                &self.client,
                &self.supported_platforms,
                self.args.threads,
            )
            .await?;

            if !tasks.is_empty() {
                info!("Fetched {} tasks.", tasks.len());
                break Ok(Return::FetchTasks(tasks));
            }

            info!("No tasks found. Sleeping before attempting again.");
            time::sleep(Duration::from_secs(15)).await;
        }
    }

    async fn execute_task(
        self: Arc<Self>,
        TaskInfo {
            task,
            file_path,
            platform_id,
        }: TaskInfo,
    ) -> ClientResult<Return> {
        let slot_dir = tempfile::tempdir()?;

        info!("Task id: {}", task.id);
        debug!("Project id: {}", task.project_id);
        debug!("Platform id: {}", platform_id);
        debug!("Slot dir: {}", slot_dir.path().display());

        let mut child = self
            .supported_platforms
            .get_command(&file_path, platform_id)
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
    let supported_platforms = SupportedPlatforms::detect(&args.cache_dir, &client).await?;

    Arc::new(ClusterizerClient {
        client,
        args,
        supported_platforms,
    })
    .run()
    .await
}
