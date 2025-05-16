use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateStateError {
    #[error("invalid assignment")]
    InvalidAssignment,
    #[error("validatestate value outside of range")]
    BadValidateState,
}
