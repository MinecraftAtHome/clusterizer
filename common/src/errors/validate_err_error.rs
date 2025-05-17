use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateErrError {
    #[error("invalid assignment")]
    InvalidAssignment,
    #[error("provided assignments needed value out of bounds")]
    AssignmentsNeededOutOfBounds,
    #[error("canonical result already set")]
    CanonicalResultExists,
    #[error("provided error assignments belong to multiple tasks")]
    ErroredRelationshipError,
    #[error("provided inconclusive assignments belong to multiple tasks")]
    InconclusiveRelationshipError,
    #[error(
        "provided inconclusive assignments belong to different tasks than the error assignments"
    )]
    RequestAssignmentsRelationshipError,
}
