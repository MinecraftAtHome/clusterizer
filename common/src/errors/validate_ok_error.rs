use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateOkError {
    #[error("invalid task")]
    InvalidTask,
    #[error("invalid result")]
    InvalidResult,
    #[error("canonical result already set")]
    CanonicalResultExists,
    #[error("provided results belong to multiple tasks")]
    ResultTaskRelationshipInconsistent,
    #[error("result count does not equal quorum")]
    ResultCountQuorumNotEqual,
    #[error("assignments which have been canceled cannot be validated")]
    AssignmentCanceledError,
}
