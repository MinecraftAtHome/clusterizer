use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateFetchError {
    #[error("invalid task")]
    InvalidTask,
    #[error("invalid project")]
    InvalidProject,
}
