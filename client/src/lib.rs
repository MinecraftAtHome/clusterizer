pub mod result;
pub mod supported_platforms;

use std::{
    collections::HashMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use clusterizer_api::client::ApiClient;
use clusterizer_common::{
    records::{File, FileFilter, Platform, ProjectRunnerFilter, Task},
    requests::FetchTasksRequest,
    types::Id,
};
use clusterizer_util::Hex;
use tokio::task::{self, JoinHandle};
use tracing::{debug, info, warn};
use zip::ZipArchive;

use crate::{result::ClientResult, supported_platforms::SupportedPlatforms};

#[derive(Debug)]
pub struct TaskInfo {
    pub task: Task,
    pub file_path: PathBuf,
    pub platform_id: Id<Platform>,
}

pub async fn fetch_tasks(
    cache_dir: &Path,
    client: &ApiClient,
    supported_platforms: &SupportedPlatforms,
    limit: usize,
) -> ClientResult<Vec<TaskInfo>> {
    let project_runners: HashMap<_, _> = client
        .get(
            &ProjectRunnerFilter::default()
                .disabled_at(vec![None])
                .platform_id(supported_platforms.platform_ids().collect::<Vec<_>>()),
        )
        .await?
        .into_iter()
        .map(|project_runner| (project_runner.project_id, project_runner))
        .collect();

    let tasks: Vec<_> = client
        .fetch_tasks(&FetchTasksRequest {
            project_ids: project_runners.keys().copied().collect(),
            limit,
        })
        .await?
        .into_iter()
        .filter_map(|task| {
            let Some(project_runner) = project_runners.get(&task.project_id) else {
                warn!(
                    "Unwanted task ({}) for project ({})",
                    task.id, task.project_id
                );

                return None;
            };

            Some((task, project_runner))
        })
        .collect();

    let file_ids: Vec<_> = tasks
        .iter()
        .map(|(_, project_runner)| project_runner.file_id)
        .collect();

    let files = download_archives(cache_dir, client, file_ids).await?;

    Ok(tasks
        .into_iter()
        .filter_map(|(task, project_runner)| {
            let Some(file_path) = files.get(&project_runner.file_id) else {
                warn!(
                    "Missing file ({}) for project runner ({})",
                    project_runner.file_id, project_runner.id
                );

                return None;
            };

            Some(TaskInfo {
                task,
                file_path: file_path.clone(),
                platform_id: project_runner.platform_id,
            })
        })
        .collect())
}

pub async fn download_archives(
    cache_dir: &Path,
    client: &ApiClient,
    file_ids: Vec<Id<File>>,
) -> ClientResult<HashMap<Id<File>, PathBuf>> {
    let binaries_dir = cache_dir.join("bin");
    let temp_dir = cache_dir.join("tmp");

    fs::create_dir_all(&binaries_dir)?;
    fs::create_dir_all(&temp_dir)?;

    let tasks: Vec<_> = client
        .get(&FileFilter::default().id(file_ids))
        .await?
        .into_iter()
        .map(|file| -> JoinHandle<ClientResult<_>> {
            let dir = binaries_dir.join(format!("{}", Hex(&file.hash)));
            let temp_dir = temp_dir.clone();

            task::spawn(async move {
                if !dir.exists() {
                    info!("Downloading archive {}", file.url);

                    let bytes = reqwest::get(&file.url)
                        .await?
                        .error_for_status()?
                        .bytes()
                        .await?;

                    let extract_dir = tempfile::tempdir_in(temp_dir)?;

                    ZipArchive::new(Cursor::new(bytes))?.extract(&extract_dir)?;
                    fs::rename(&extract_dir, &dir)?;
                } else {
                    debug!("Archive {} was cached", file.url);
                }

                Ok((file.id, dir))
            })
        })
        .collect();

    let mut files = HashMap::new();

    for task in tasks {
        let (file_id, dir) = task.await??;
        files.insert(file_id, dir);
    }

    Ok(files)
}
