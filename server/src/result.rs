use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use clusterizer_common::errors::{
    FetchTasksError, Infallible, NotFound, RegisterError, SubmitResultError
};
use serde::Serialize;

trait Status {
    fn status(&self) -> StatusCode;
}

impl Status for Infallible {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl Status for NotFound {
    fn status(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

impl Status for RegisterError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl Status for FetchTasksError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidProject => StatusCode::NOT_FOUND,
        }
    }
}

impl Status for SubmitResultError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidTask => StatusCode::NOT_FOUND,
            Self::AlreadyExists => StatusCode::BAD_REQUEST,
        }
    }
}

pub enum AppError<E> {
    Specific(E),
    Generic,
}

pub type AppResult<T, E> = Result<T, AppError<E>>;

impl<T> From<T> for AppError<T> {
    fn from(err: T) -> Self {
        Self::Specific(err)
    }
}

impl From<sqlx::Error> for AppError<Infallible> {
    fn from(_: sqlx::Error) -> Self {
        Self::Generic
    }
}

impl From<sqlx::Error> for AppError<NotFound> {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::Specific(NotFound),
            _ => Self::Generic,
        }
    }
}

impl From<sqlx::Error> for AppError<RegisterError> {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(err) if err.constraint() == Some("users_name_key") => {
                Self::Specific(RegisterError::AlreadyExists)
            }
            _ => Self::Generic,
        }
    }
}

impl From<sqlx::Error> for AppError<FetchTasksError> {
    fn from(_: sqlx::Error) -> Self {
        Self::Generic
    }
}

impl From<sqlx::Error> for AppError<SubmitResultError> {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::Specific(SubmitResultError::InvalidTask),
            sqlx::Error::Database(err) if err.is_unique_violation() => {
                Self::Specific(SubmitResultError::AlreadyExists)
            }
            _ => Self::Generic,
        }
    }
}

impl<E: Serialize + Status> IntoResponse for AppError<E> {
    fn into_response(self) -> Response {
        match self {
            Self::Specific(err) => (err.status(), Json(err)).into_response(),
            Self::Generic => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
