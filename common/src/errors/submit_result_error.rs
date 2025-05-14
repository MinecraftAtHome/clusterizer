use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum SubmitResultError {
    #[error("invalid task")]
    InvalidTask,
    #[error("already exists")]
    AlreadyExists,
}
