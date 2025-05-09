use std::{
    env,
    ffi::OsString,
    fs,
    io::Cursor,
    iter::{self, Empty},
    path::PathBuf,
    thread,
    time::Duration,
};

use clusterizer_common::{messages::SubmitRequest, types::Task};
use tokio::process::Command;
use zip::ZipArchive;

use crate::{args::Args, result::ClientResult};

pub struct ClusterizerClient {
    api_client: clusterizer_api::Client,
    data_path: PathBuf,
    platform_id: i64,
}

impl From<Args> for ClusterizerClient {
    fn from(args: Args) -> Self {
        ClusterizerClient {
            api_client: clusterizer_api::Client::new(args.server_url, args.api_key),
            data_path: args.data_path,
            platform_id: args.platform_id,
        }
    }
}

impl ClusterizerClient {
    pub async fn run(&self) -> ClientResult<()> {
        loop {
            let tasks = self.api_client.fetch_tasks(self.platform_id).await?;

            if tasks.is_empty() {
                println!("No tasks found. Sleeping before attempting again.");
                thread::sleep(Duration::from_millis(15000));
            } else {
                for task in tasks {
                    if let Err(err) = self.execute_task(&task).await {
                        eprintln!("Error: {err}");
                    }
                }
            }
        }
    }

    async fn execute_task(&self, task: &Task) -> ClientResult<()> {
        let project = self.api_client.get_project(task.project_id).await?;
        let project_version = self
            .api_client
            .get_project_project_version(project.id, self.platform_id)
            .await?;
        let slot_path = self.data_path.join("slots").join(format!("{}", task.id));

        println!("Task id: {}, stdin: {}", task.id, task.stdin);
        println!("Project id: {}, name: {}", project.id, project.name);
        println!(
            "Project version id: {}, archive url: {}",
            project_version.id, project_version.archive_url
        );
        println!("Slot path: {}", slot_path.display());

        fs::create_dir_all(&slot_path)?;

        let response = reqwest::get(project_version.archive_url)
            .await?
            .error_for_status()?;
        let bytes = response.bytes().await?;

        ZipArchive::new(Cursor::new(bytes))?.extract(&slot_path)?;

        let program = slot_path
            .join(format!("main{}", env::consts::EXE_SUFFIX))
            .canonicalize()?;
        let args: Empty<OsString> = iter::empty();

        let output = Command::new(program)
            .args(args)
            .current_dir(slot_path)
            .output()
            .await?;

        let result = SubmitRequest {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        };

        self.api_client.submit_task(task.id, &result).await?;

        Ok(())
    }
}
