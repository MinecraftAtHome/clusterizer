use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum CreateFileError {
    #[error("forbidden")]
    Forbidden,
    #[error("url is invalid")]
    InvalidUrl,
}
