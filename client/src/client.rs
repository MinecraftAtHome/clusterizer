use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self},
    io, iter,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use clusterizer_api::Client as ApiClient;
use clusterizer_common::{messages::SubmitRequest, types::Task};
use thiserror::Error;
use tokio::process::Command;
use url::{ParseError, Url};
use zip::result::ZipError;

use crate::{args::Args, util};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ClientError {
    Reqwest(#[from] reqwest::Error),
    Zip(#[from] ZipError),
    Io(#[from] io::Error),
    Url(#[from] ParseError),
    #[error("bad archive url")]
    BadArchiveUrl,
}

pub struct ClusterizerClient {
    api_client: ApiClient,
    data_dir: PathBuf,
    platform_id: i64,
}

impl From<Args> for ClusterizerClient {
    fn from(args: Args) -> Self {
        ClusterizerClient {
            api_client: ApiClient::new(args.server_url, args.api_key),
            data_dir: args.data_dir,
            platform_id: args.platform_id,
        }
    }
}

impl ClusterizerClient {
    pub async fn run(&self) -> Result<(), ClientError> {
        loop {
            let tasks = self.api_client.fetch_tasks().await?;
            if tasks.is_empty() {
                eprintln!("No tasks found. Sleeping before attempting again.");
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
    async fn execute_task(&self, task: &Task) -> Result<(), ClientError> {
        let project = self.api_client.get_project(task.project_id).await?;
        let project_versions = self
            .api_client
            .get_project_project_versions(project.id)
            .await?;

        println!("Task id: {}\t Task stdin: {}", task.id, task.stdin);
        println!("Projectid: {}\t Project name: {}", project.id, project.name);

        let slot_path = self.data_dir.join("slots").join(format!("{}", task.id));

        fs::create_dir_all(&slot_path)?;

        let archive_url = Url::parse(&project_versions[0].archive_url)?;
        let archive_name = archive_url
            .path_segments()
            .and_then(Iterator::last)
            .ok_or(ClientError::BadArchiveUrl)?;
        let download_path = slot_path.join(archive_name);

        self.prepare_slot(&slot_path, &download_path, &project_versions[0].archive_url)
            .await?;

        let result_data = self
            .run_program(
                &slot_path,
                &slot_path.join(format!("main{}", env::consts::EXE_SUFFIX)),
                iter::empty::<OsString>(),
            )
            .await?;

        self.api_client.submit_task(task.id, &result_data).await?;
        Ok(())
    }

    pub async fn prepare_slot(
        &self,
        slot_path: &Path,
        download_path: &Path,
        archive_url: &str,
    ) -> Result<(), ClientError> {
        let response = reqwest::get(archive_url).await?.error_for_status()?;
        let body = response.bytes().await?;
        fs::write(download_path, body)?;
        util::zip_extract(download_path, slot_path)?;
        Ok(())
    }

    pub async fn run_program<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        current_dir: &Path,
        program: &Path,
        args: I,
    ) -> Result<SubmitRequest, ClientError> {
        let output = Command::new(fs::canonicalize(program)?)
            .args(args)
            .current_dir(current_dir)
            .output()
            .await?;

        Ok(SubmitRequest {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        })
    }
}
