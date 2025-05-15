use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateSubmitError {
    #[error("invalid assignment")]
    InvalidAssignment,
    #[error("invalid assignment state")]
    InvalidAssignmentState,
    #[error("provided assignments needed value out of bounds")]
    AssignmentsNeededOutOfBounds,
    #[error("result count is less than quorum")]
    ResultCountLessThanQuorum,
    #[error("state transition forbidden")]
    StateTransitionForbidden,
}
