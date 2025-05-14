use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum RegisterError {
    #[error("too short")]
    TooShort,
    #[error("too long")]
    TooLong,
    #[error("invalid character")]
    InvalidCharacter,
    #[error("already exists")]
    AlreadyExists,
}
