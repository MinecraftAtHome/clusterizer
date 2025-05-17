use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use clusterizer_common::errors::{
    FetchTasksError, Infallible, NotFound, RegisterError, SubmitResultError, ValidateErrError,
    ValidateFetchError, ValidateOkError,
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

impl Status for ValidateFetchError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidProject => StatusCode::NOT_FOUND,
        }
    }
}

impl Status for ValidateOkError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidAssignment => StatusCode::NOT_FOUND,
            Self::CanonicalResultExists => StatusCode::BAD_REQUEST,
            Self::AssignmentCanceledError => StatusCode::BAD_REQUEST,
            Self::ResultCountQuorumNotEqual => StatusCode::BAD_REQUEST,
            Self::AssignmentTaskRelationshipError => StatusCode::BAD_REQUEST,
        }
    }
}

impl Status for ValidateErrError {
    fn status(&self) -> StatusCode {
        match self {
            Self::AssignmentsNeededOutOfBounds => StatusCode::BAD_REQUEST,
            Self::InvalidAssignment => StatusCode::NOT_FOUND,
            Self::CanonicalResultExists => StatusCode::BAD_REQUEST,
            Self::AssignmentTaskRelationshipError => StatusCode::BAD_REQUEST,
            Self::RequestAssignmentsRelationshipError => StatusCode::BAD_REQUEST
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

impl From<sqlx::Error> for AppError<ValidateFetchError> {
    fn from(_: sqlx::Error) -> Self {
        Self::Generic
    }
}

impl From<sqlx::Error> for AppError<ValidateOkError> {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::Specific(ValidateOkError::InvalidAssignment),
            _ => Self::Generic,
        }
    }
}
impl From<sqlx::Error> for AppError<ValidateErrError> {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::Specific(ValidateErrError::InvalidAssignment),
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
