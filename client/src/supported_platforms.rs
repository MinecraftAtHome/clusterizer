use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::Stdio,
};

use clusterizer_api::client::ApiClient;
use clusterizer_common::{
    records::{Platform, PlatformFilter, PlatformRunnerFilter},
    types::Id,
};
use tokio::{
    process::Command,
    task::{self, JoinHandle},
};
use tracing::{info, warn};

use crate::result::ClientResult;

#[derive(Debug, Clone)]
enum PlatformStrategy {
    Native,
    Wrapper {
        file_path: PathBuf,
        platform_id: Id<Platform>,
    },
}

#[derive(Debug, Clone)]
pub struct SupportedPlatforms {
    platform_strategies: HashMap<Id<Platform>, PlatformStrategy>,
}

impl SupportedPlatforms {
    fn new() -> Self {
        Self {
            platform_strategies: HashMap::new(),
        }
    }

    fn insert(&mut self, platform_id: Id<Platform>, strategy: PlatformStrategy) {
        self.platform_strategies.insert(platform_id, strategy);
    }

    pub fn platform_ids(&self) -> impl Iterator<Item = Id<Platform>> {
        self.platform_strategies.keys().copied()
    }

    pub fn get_command(&self, file_path: &Path, platform_id: Id<Platform>) -> Command {
        self.get_command_with_strategy(file_path, &self.platform_strategies[&platform_id])
    }

    fn get_command_with_strategy(&self, file_path: &Path, strategy: &PlatformStrategy) -> Command {
        match strategy {
            PlatformStrategy::Native => {
                Command::new(file_path.join(format!("main{}", env::consts::EXE_SUFFIX)))
            }
            PlatformStrategy::Wrapper {
                file_path: wrapper_file_path,
                platform_id: wrapper_platform_id,
            } => {
                let mut command = self.get_command(wrapper_file_path, *wrapper_platform_id);
                command.arg(file_path);
                command
            }
        }
    }

    pub async fn detect(cache_dir: &Path, client: &ApiClient) -> ClientResult<Self> {
        let mut strategy_queue = vec![PlatformStrategy::Native];
        let mut supported_platforms = Self::new();
        let mut platforms = client.get(&PlatformFilter::default()).await?;

        let files = crate::download_archives(
            cache_dir,
            client,
            platforms.iter().map(|platform| platform.file_id).collect(),
        )
        .await?;

        while !strategy_queue.is_empty() {
            let tasks: Vec<_> = platforms
                .drain(..)
                .map(|platform| -> JoinHandle<ClientResult<_>> {
                    let strategy_queue = strategy_queue.clone();
                    let supported_platforms = supported_platforms.clone();
                    let files = files.clone();

                    task::spawn(async move {
                        for strategy in strategy_queue {
                            let Some(file_path) = files.get(&platform.file_id) else {
                                warn!(
                                    "Missing file ({}) for platform ({})",
                                    platform.file_id, platform.id
                                );

                                continue;
                            };

                            let slot_dir = tempfile::tempdir()?;

                            if supported_platforms
                                .get_command_with_strategy(file_path, &strategy)
                                .current_dir(&slot_dir)
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .status()
                                .await
                                .is_ok_and(|status| status.success())
                            {
                                return Ok((platform, Some(strategy)));
                            }
                        }

                        Ok((platform, None))
                    })
                })
                .collect();

            let mut new_platforms = Vec::new();

            for task in tasks {
                let (platform, strategy) = task.await??;

                if let Some(strategy) = strategy {
                    info!("Supported platform: {}", platform.name);
                    supported_platforms.insert(platform.id, strategy);
                    new_platforms.push(platform.id);
                } else {
                    platforms.push(platform);
                }
            }

            let new_platform_runners = client
                .get(&PlatformRunnerFilter::default().platform_id(new_platforms))
                .await?;

            let mut files = crate::download_archives(
                cache_dir,
                client,
                new_platform_runners
                    .iter()
                    .map(|platform_runner| platform_runner.file_id)
                    .collect(),
            )
            .await?;

            strategy_queue.splice(
                ..,
                new_platform_runners
                    .into_iter()
                    .filter_map(|platform_runner| {
                        let Some(file_path) = files.remove(&platform_runner.file_id) else {
                            warn!(
                                "Missing file ({}) for platform runner ({})",
                                platform_runner.file_id, platform_runner.id
                            );

                            return None;
                        };

                        Some(PlatformStrategy::Wrapper {
                            file_path,
                            platform_id: platform_runner.platform_id,
                        })
                    }),
            );
        }

        Ok(supported_platforms)
    }
}
