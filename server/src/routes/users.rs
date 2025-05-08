use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
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

pub async fn get_one(State(state): State<AppState>, Path(user_id): Path<i64>) -> ApiResult<User> {
    Ok(Json(
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn get_profile(State(state): State<AppState>, Auth(user_id): Auth) -> ApiResult<User> {
    Ok(Json(
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> ApiResult<RegisterResponse> {
    if request.name.len() < 3 {
        Err(StatusCode::BAD_REQUEST)?;
    }

    if request.name.len() > 32 {
        Err(StatusCode::BAD_REQUEST)?;
    }

    if !request
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        Err(StatusCode::BAD_REQUEST)?;
    }

    let user = sqlx::query!(
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
        api_key: auth::api_key(&state, user.id),
    }))
}
