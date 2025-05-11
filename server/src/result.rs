use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct ApiError(StatusCode);

pub type ApiResult<T> = Result<Json<T>, ApiError>;

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self(match error {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            sqlx::Error::Database(ref err) if err.constraint() == Some("users_name_key") => {
                StatusCode::BAD_REQUEST
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
    }
}

impl From<StatusCode> for ApiError {
    fn from(status_code: StatusCode) -> Self {
        Self(status_code)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}
