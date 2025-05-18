use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use status::Status;

mod status;

pub trait ResultExt<T> {
    fn map_not_found<E>(self, error: E) -> AppResult<T, E>;
    fn map_unique_violation<E>(self, error: E) -> AppResult<T, E>;
}

pub enum AppError<E> {
    Specific(E),
    Sqlx,
}

pub type AppResult<T, E> = Result<T, AppError<E>>;

impl<E> From<sqlx::Error> for AppError<E> {
    fn from(_: sqlx::Error) -> Self {
        Self::Sqlx
    }
}

impl<T> ResultExt<T> for Result<T, sqlx::Error> {
    fn map_not_found<E>(self, error: E) -> AppResult<T, E> {
        self.map_err(|err| match err {
            sqlx::Error::RowNotFound => AppError::Specific(error),
            _ => AppError::Sqlx,
        })
    }

    fn map_unique_violation<E>(self, error: E) -> AppResult<T, E> {
        self.map_err(|err| match err {
            sqlx::Error::Database(err) if err.is_unique_violation() => AppError::Specific(error),
            _ => AppError::Sqlx,
        })
    }
}

impl<E: Serialize + Status> IntoResponse for AppError<E> {
    fn into_response(self) -> Response {
        match self {
            Self::Specific(err) => (err.status(), Json(err)).into_response(),
            Self::Sqlx => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
