use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    messages::{RegisterRequest, RegisterResponse},
    types::User,
};

use crate::{
    auth::{self, Auth},
    result::ApiResult,
    state::AppState,
};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<User>> {
    Ok(Json(
        sqlx::query_as!(User, "SELECT * FROM users")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<User> {
    Ok(Json(
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> ApiResult<RegisterResponse> {
    // TODO: sanitize name
    let record = sqlx::query!(
        "
        INSERT INTO users (
            name
        ) VALUES (
            $1
        )
        RETURNING id
        ",
        request.name
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(RegisterResponse {
        api_key: auth::api_key(&state, record.id),
    }))
}

pub async fn profile(State(state): State<AppState>, Auth(id): Auth) -> ApiResult<User> {
    Ok(Json(
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(&state.pool)
            .await?,
    ))
}
