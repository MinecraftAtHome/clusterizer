use axum::{http::StatusCode, Json};
use clusterizer_common::error::Error as ClusterizerError;
use tokio_postgres::error::Error as PgError;

pub type Error = (StatusCode, Json<ClusterizerError>);
pub type Result<T> = std::result::Result<(StatusCode, Json<T>), Error>;

pub fn error(err: ClusterizerError) -> Error {
    (StatusCode::BAD_REQUEST, Json(err))
}

pub fn pg_error(err: PgError) -> Error {
    err.as_db_error()
        .map(|err| match err.constraint() {
            Some("users_name_key") => error(ClusterizerError::UsernameTaken),
            _ => error(ClusterizerError::Unknown),
        })
        .unwrap_or_else(|| error(ClusterizerError::Unknown))
}
