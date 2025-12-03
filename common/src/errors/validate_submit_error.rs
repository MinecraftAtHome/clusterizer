use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateSubmitError {
    #[error("invalid result")]
    InvalidResult,
    #[error("expected results for exactly one task")]
    InvalidTaskCount,
    #[error("the group id of all results in a group must be the first submitted result")]
    InconsistentGroup,
    #[error("forbidden state transition")]
    ForbiddenStateTransition,
    #[error("missing some results for this task")]
    MissingResults,
}
