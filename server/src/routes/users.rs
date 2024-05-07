use crate::app::App;
use crate::error::{error, Result};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use clusterizer_common::{
    error::Error as ClusterizerError,
    types::{CreateUser, CreateUserResponse},
};
use std::sync::Arc;

pub fn router() -> Router<Arc<App>> {
    Router::new().route("/", post(create))
}

async fn create(
    State(app): State<Arc<App>>,
    Json(create_user): Json<CreateUser>,
) -> Result<CreateUserResponse> {
    if create_user.name.len() > 32 {
        return Err(error(ClusterizerError::UsernameTooLong));
    }

    let id = app
        .query_one(
            "INSERT INTO users (name) VALUES ($1) RETURNING id",
            &[&create_user.name],
        )
        .await?;

    Ok((StatusCode::CREATED, Json(CreateUserResponse { id })))
}
