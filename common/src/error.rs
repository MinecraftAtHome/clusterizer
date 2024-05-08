use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "error")]
pub enum Error {
    Unknown,
    UsernameTooLong,
    UsernameTooShort,
    UsernameInvalidChar,
    UsernameTaken,
}
