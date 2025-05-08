use std::{
    error,
    ffi::OsStr,
    fmt,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use clusterizer_api::Client as ApiClient;
use clusterizer_common::{
    messages::SubmitRequest,
    types::{Project, ProjectVersion, Task},
};
use thiserror::Error;
use tokio::process::Command;
use url::ParseError;
use zip::{ZipArchive, result::ZipError};

#[derive(Error, Debug)]
enum GetProgramError {
    #[error("client failed to find main executable in task archive")]
    NoBinaryFound,
    #[error("client failed to find task archive from given url")]
    NoArchiveFound,
    #[error("server did not reply, url is possibly bad")]
    BadURL,
}

#[derive(Error, Debug)]
#[error(transparent)]
struct RunProgramError(#[from] std::io::Error);
#[derive(Debug)]
pub enum ClientError {
    Reqwest(reqwest::Error),
    Zip(ZipError),
    Io(io::Error),
    Url(ParseError),
}
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Reqwest(e) => write!(f, "Reqwest error: {e}"),
            ClientError::Zip(e) => write!(f, "Zip error: {e}"),
            ClientError::Io(e) => write!(f, "IO error: {e}"),
            ClientError::Url(e) => write!(f, "failed to parse url: {e}"),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ClientError::Reqwest(e) => Some(e),
            ClientError::Zip(e) => Some(e),
            ClientError::Io(e) => Some(e),
            ClientError::Url(e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        ClientError::Reqwest(e)
    }
}

impl From<ZipError> for ClientError {
    fn from(e: ZipError) -> Self {
        ClientError::Zip(e)
    }
}

impl From<io::Error> for ClientError {
    fn from(e: io::Error) -> Self {
        ClientError::Io(e)
    }
}
impl From<ParseError> for ClientError {
    fn from(e: ParseError) -> Self {
        ClientError::Url(e)
    }
}
pub struct ClusterizerClient {
    api_client: ApiClient,
    pub data_dir: PathBuf,
}

impl ClusterizerClient {
    pub fn new(api_key: Option<String>, url: String, data_dir: PathBuf) -> ClusterizerClient {
        ClusterizerClient {
            api_client: ApiClient::new(url, api_key),
            data_dir: data_dir,
        }
    }
    pub async fn fetch_tasks(&self) -> Result<Vec<Task>, ClientError> {
        Ok(self.api_client.fetch_tasks().await?)
    }

    pub async fn submit_task(
        &self,
        task_id: i64,
        submit_request: &SubmitRequest,
    ) -> Result<(), ClientError> {
        Ok(self.api_client.submit_task(task_id, submit_request).await?)
    }

    pub async fn get_project(&self, project_id: i64) -> Result<Project, ClientError> {
        Ok(self.api_client.get_project(project_id).await?)
    }

    pub async fn get_project_version(
        &self,
        project_version_id: i64,
    ) -> Result<ProjectVersion, ClientError> {
        Ok(self
            .api_client
            .get_project_version(project_version_id)
            .await?)
    }

    fn zip_extract(archive_file: &Path, target_dir: &Path) -> Result<(), ClientError> {
        let file = File::open(archive_file)?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(target_dir)?;
        Ok(())
    }

    pub async fn get_program(
        &self,
        download_path: &Path,
        slot_path: &Path,
        archive_url: &str,
    ) -> Result<PathBuf, ClientError> {
        let resp = reqwest::get(archive_url).await?;
        let body = resp.bytes().await?;
        fs::write(download_path, body);
        Self::zip_extract(download_path, slot_path)?;
        let mut bin_name = PathBuf::new();
        match fs::read_dir(slot_path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if entry
                                .path()
                                .to_str()
                                .expect("Failed to convert to string")
                                .ends_with(".bin")
                            {
                                bin_name = PathBuf::from(
                                    entry.path().to_str().expect("Failed to convert to string"),
                                )
                                .to_path_buf();
                            }
                        }
                        Err(e) => eprintln!("Error: {e}"),
                    }
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        }

        Ok(bin_name)
    }

    pub async fn run_program<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        slot_path: &Path,
        prog_argc: I,
        prog_name: &Path,
    ) -> Result<SubmitRequest, ClientError> {
        let abs_path = fs::canonicalize(prog_name)?;
        let output = Command::new(abs_path)
            .args(prog_argc)
            .current_dir(slot_path)
            .output()
            .await?;

        let exit_code = output.status.code().unwrap_or(-100);

        let result_data = SubmitRequest {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
        };
        Ok(result_data)
    }
}
