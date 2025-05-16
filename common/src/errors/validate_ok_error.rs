use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateOkError {
    #[error("invalid task")]
    InvalidTask,
    #[error("canonical result already set")]
    CanonicalResultExists,
}
