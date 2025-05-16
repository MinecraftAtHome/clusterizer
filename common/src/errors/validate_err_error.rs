use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateErrError {
    #[error("invalid task")]
    InvalidTask,
    #[error("provided assignments needed value out of bounds")]
    AssignmentsNeededOutOfBounds,
    #[error("canonical result already set")]
    CanonicalResultExists,
}
