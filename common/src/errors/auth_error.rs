use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum AuthRejection {
    #[error("unrecognized api key provided")]
    BadAPIKey,
    #[error("your user has been disabled")]
    UserDisabled,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        match self {
            Self::BadAPIKey => (StatusCode::BAD_REQUEST, "Bad API Key provided").into_response(),
            Self::UserDisabled => (StatusCode::UNAUTHORIZED, "User is disabled.").into_response(),
        }
    }
}
