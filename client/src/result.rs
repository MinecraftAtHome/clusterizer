use std::{error::Error, io};

use clusterizer_api::result::ApiError;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ClientError {
    Specific(Box<dyn Error>),
    Reqwest(#[from] reqwest::Error),
    Zip(#[from] ZipError),
    Io(#[from] io::Error),
    #[error("project version not found")]
    ProjectVersionNotFound,
}

pub type ClientResult<T> = Result<T, ClientError>;

impl<E: Error + 'static> From<ApiError<E>> for ClientError {
    fn from(err: ApiError<E>) -> Self {
        match err {
            ApiError::Specific(err) => Self::Specific(Box::new(err)),
            ApiError::Reqwest(err) => Self::Reqwest(err),
        }
    }
}
