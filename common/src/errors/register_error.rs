use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum RegisterError {
    #[error("name too short")]
    TooShort,
    #[error("name too long")]
    TooLong,
    #[error("name includes invalid character")]
    InvalidCharacter,
    #[error("user already exists with that name")]
    AlreadyExists,
}
