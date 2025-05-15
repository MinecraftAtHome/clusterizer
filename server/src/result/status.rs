use axum::http::StatusCode;
use clusterizer_common::errors::{
    FetchTasksError, Infallible, NotFound, RegisterError, SubmitResultError, ValidateFetchError,
    ValidateSubmitError,
};

pub trait Status {
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
        StatusCode::BAD_REQUEST
    }
}

impl Status for SubmitResultError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl Status for ValidateSubmitError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl Status for ValidateFetchError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
