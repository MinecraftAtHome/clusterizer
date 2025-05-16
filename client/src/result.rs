use std::{error::Error, io};

use clusterizer_api::result::ApiError;
use thiserror::Error;
use tokio::task::JoinError;
use zip::result::ZipError;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ClientError {
    Specific(Box<dyn Error + Sync + Send>),
    Reqwest(#[from] reqwest::Error),
    Zip(#[from] ZipError),
    Io(#[from] io::Error),
    Join(#[from] JoinError),
}

pub type ClientResult<T> = Result<T, ClientError>;

impl<E: Error + Sync + Send + 'static> From<ApiError<E>> for ClientError {
    fn from(err: ApiError<E>) -> Self {
        match err {
            ApiError::Specific(err) => Self::Specific(Box::new(err)),
            ApiError::String(err) => Self::Specific(err.into()),
            ApiError::Reqwest(err) => Self::Reqwest(err),
        }
    }
}
