use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    iter::{self, Empty},
    thread,
    time::Duration,
};

use clusterizer_api::client::ApiClient;
use clusterizer_common::{
    requests::{FetchTasksRequest, SubmitResultRequest},
    types::{ProjectVersion, Task},
};
use log::{debug, error, info};
use tokio::process::Command;
use zip::ZipArchive;

use crate::{
    args::RunArgs,
    result::{ClientError, ClientResult},
};

pub struct ClusterizerClient {
    client: ApiClient,
    args: RunArgs,
}

impl ClusterizerClient {
    pub fn new(client: ApiClient, args: RunArgs) -> ClusterizerClient {
        ClusterizerClient { client, args }
    }

    pub async fn run(&self) -> ClientResult<()> {
        fs::create_dir_all(&self.args.cache_dir)?;

        loop {
            // TODO: cache project versions instead of fetching them each time. the cache can also
            // be used in execute_task when trying to find the right binary to use for a specific
            // task.

            let mut project_ids: Vec<_> = self
                .client
                .get_all_by::<ProjectVersion, _>(self.args.platform_id)
                .await?
                .into_iter()
                .filter(|project_version| project_version.disabled_at.is_none())
                .map(|project_version| project_version.project_id)
                .collect();

            project_ids.sort();
            project_ids.dedup();

            let tasks = self
                .client
                .fetch_tasks(&FetchTasksRequest { project_ids })
                .await?;

            if tasks.is_empty() {
                info!("No tasks found. Sleeping before attempting again.");
                thread::sleep(Duration::from_millis(15000));
            } else {
                for task in tasks {
                    if let Err(err) = self.execute_task(&task).await {
                        error!("Error: {}.", err);
                    }
                }
            }
        }
    }

    async fn execute_task(&self, task: &Task) -> ClientResult<()> {
        let project = self.client.get_one(task.project_id).await?;
        let project_version = self
            .client
            .get_all_by::<ProjectVersion, _>(task.project_id)
            .await?
            .into_iter()
            .filter(|project_version| project_version.disabled_at.is_none())
            .find(|project_version| project_version.platform_id == self.args.platform_id)
            .ok_or(ClientError::ProjectVersionNotFound)?;

        let slot_dir = tempfile::tempdir()?;
        let slot_path = slot_dir.path();

        info!("Task id: {}, stdin: {}", task.id, task.stdin);
        info!("Project id: {}, name: {}", project.id, project.name);
        debug!(
            "Project version id: {}, archive url: {}",
            project_version.id, project_version.archive_url
        );
        debug!("Slot path: {}", slot_path.display());

        fs::create_dir_all(slot_path)?;

        let archive_cache_path = &self
            .args
            .cache_dir
            .join(project_version.id.to_string() + ".zip");

        if archive_cache_path.is_file() {
            debug!("Archive {} was cached.", archive_cache_path.display());
        } else {
            debug!("Archive {} is not cached.", archive_cache_path.display());

            let bytes = reqwest::get(project_version.archive_url)
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            fs::write(archive_cache_path, &bytes)?;
        }

        ZipArchive::new(File::open(archive_cache_path)?)?.extract(slot_path)?;

        let program = slot_path
            .join(format!("main{}", env::consts::EXE_SUFFIX))
            .canonicalize()?;
        let args: Empty<OsString> = iter::empty();

        let output = Command::new(program)
            .args(args)
            .current_dir(slot_path)
            .output()
            .await?;

        let result = SubmitResultRequest {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        };

        self.client.submit_result(task.id, &result).await?;

        Ok(())
    }
}
