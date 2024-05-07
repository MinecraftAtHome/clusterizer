use clusterizer_common::error::Error as ClusterizerError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    ClusterizerError(ClusterizerError),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}
