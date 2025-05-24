use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateSubmitError {
    #[error("invalid assignment")]
    InvalidAssignment,
    #[error("task already validated and this result is not valid")]
    InconsistentValidationState,
    #[error("multi-task validation in a single request is currently not implemented")]
    MultipleTasksDisallowed,
    #[error("too many groups meeting quorum were provided")]
    ValidityAmbiguous,
    #[error("state transition forbidden")]
    StateTransitionForbidden,
}
