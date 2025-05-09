use std::io;

use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ClientError {
    Reqwest(#[from] reqwest::Error),
    Zip(#[from] ZipError),
    Io(#[from] io::Error),
    #[error("no command given")]
    NoCommand,
    #[error("registration failed")]
    RegistrationError,
}

pub type ClientResult<T> = Result<T, ClientError>;
