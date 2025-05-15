use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateOkError {
    #[error("invalid task")]
    InvalidTask,
    #[error("canonical result already set")]
    CanonicalResultExists
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateErrError {
    #[error("invalid task")]
    InvalidTask,
    #[error("provided assignments needed value out of bounds")]
    AssignmentsNeededOutOfBounds
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateFetchError {
    #[error("invalid task")]
    InvalidTask,
    #[error("invalid project")]
    InvalidProject
}