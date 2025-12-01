use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateSubmitError {
    #[error("invalid result given")]
    InvalidResult,
    #[error("validation group contained results belonging to multiple tasks")]
    InvalidTaskCount,
    #[error("results referred to by group id cannot refer to an result other than itself")]
    NondeterministicGroup,
    #[error("state transition forbidden")]
    ForbiddenStateTransition,
}
