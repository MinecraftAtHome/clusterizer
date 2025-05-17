use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateOkError {
    #[error("invalid assignment")]
    InvalidAssignment,
    #[error("canonical result already set")]
    CanonicalResultExists,
    #[error("provided assignments belong to multiple tasks")]
    AssignmentTaskRelationshipError,
    #[error("result count does not equal quorum")]
    ResultCountQuorumNotEqual,
    #[error("assignments which have been canceled cannot be validated")]
    AssignmentCanceledError,
}
