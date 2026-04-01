use std::{collections::HashMap, error::Error, time::Duration};

use clap::Parser;
use clusterizer_api::{client::ApiClient, result::ApiError};
use clusterizer_common::{
    records::{Assignment, AssignmentFilter, Project, Result, ResultFilter},
    requests::ValidateSubmitRequest,
    types::{AssignmentState, Id, ResultState},
};
use thiserror::Error;
use tokio::time;
use tracing::{debug, info, warn};

#[derive(Debug, Parser)]
struct ValidatorArgs {
    #[arg(long, short, default_value = "https://clusterizer.mcathome.dev")]
    server_url: String,
    #[arg(long, short)]
    api_key: Option<String>,
    #[arg(long, short)]
    project_id: Id<Project>,
}

#[derive(Debug, Error)]
#[error(transparent)]
enum ValidatorError {
    Specific(Box<dyn Error + Sync + Send>),
    Reqwest(#[from] reqwest::Error),
}

type ValidatorResult<T> = std::result::Result<T, ValidatorError>;

impl<E: Error + Sync + Send + 'static> From<ApiError<E>> for ValidatorError {
    fn from(err: ApiError<E>) -> Self {
        match err {
            ApiError::Specific(err) => Self::Specific(Box::new(err)),
            ApiError::String(err) => Self::Specific(err.into()),
            ApiError::Reqwest(err) => Self::Reqwest(err),
        }
    }
}

struct Group {
    group_id: Id<Result>,
    result_ids: Vec<Id<Result>>,
}

#[tokio::main]
async fn main() -> ValidatorResult<()> {
    tracing_subscriber::fmt::init();

    let args = ValidatorArgs::parse();
    let client = ApiClient::new(args.server_url, args.api_key);

    loop {
        let tasks = client.validate_fetch(args.project_id).await?;

        if tasks.is_empty() {
            info!("No tasks to validate.");
            time::sleep(Duration::from_secs(60)).await;
        }

        for task in tasks {
            let mut result_groups = HashMap::new();
            let mut groups = HashMap::new();

            for assignment in client
                .get_all::<Assignment>(
                    &AssignmentFilter::default()
                        .task_id(task.id)
                        .state(AssignmentState::Submitted),
                )
                .await?
            {
                let results = client
                    .get_all::<Result>(&ResultFilter::default().assignment_id(assignment.id))
                    .await?;

                if results.len() > 1 {
                    warn!("Assignment {} has multiple results.", assignment.id);
                }

                for result in results {
                    if result.state != ResultState::Error {
                        if result.exit_code != Some(0) {
                            result_groups.insert(result.id, None);
                        } else {
                            let group = groups
                                .entry(result.stdout)
                                .and_modify(|group: &mut Group| {
                                    group.group_id = group.group_id.min(result.id);
                                })
                                .or_insert_with(|| Group {
                                    group_id: result.id,
                                    result_ids: Vec::new(),
                                });

                            if result.state == ResultState::Init {
                                group.result_ids.push(result.id);
                            }
                        }
                    }
                }
            }

            for group in groups.values() {
                for &result_id in &group.result_ids {
                    result_groups.insert(result_id, Some(group.group_id));
                }
            }

            info!(
                "Validating {} result(s) for task {} with {} groups.",
                result_groups.len(),
                task.id,
                groups.len()
            );

            debug!("{:?}", result_groups);

            client
                .validate_submit(&ValidateSubmitRequest {
                    results: result_groups,
                })
                .await?;
        }
    }
}
