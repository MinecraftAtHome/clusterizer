use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateSubmitError {
    #[error("invalid result given")]
    InvalidResult,
    #[error("validation group contained results belonging to multiple tasks")]
    InvalidTaskCount,
    #[error("results referred to by group id cannot refer to an result other than itself")]
    InvalidGroupReference,
    #[error("state transition forbidden")]
    StateTransitionForbidden,
    #[error(
        "cannot attempt validation without all results relevant to choosing the canonical result"
    )]
    MissingResults,
    #[error(
        "validator must choose the earliest group_result_id by created_at date to use for the group"
    )]
    NondeterministicGroup,
}
