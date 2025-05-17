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
    #[error("provided assignmennts belong to multiple tasks")]
    AssignmentTaskRelationshipError,
    #[error("provided inconclusive assignments belong to different tasks than the error assignments")]
    RequestAssignmentsRelationshipError,
}
