use std::io;

use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ClientError {
    Reqwest(#[from] reqwest::Error),
    Zip(#[from] ZipError),
    Io(#[from] io::Error),
    #[error("project version not found")]
    ProjectVersionNotFound,
}

pub type ClientResult<T> = Result<T, ClientError>;
