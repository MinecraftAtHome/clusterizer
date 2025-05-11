use axum::{Json, extract::State, http::StatusCode};
use clusterizer_common::{
    id::Id,
    messages::{RegisterRequest, RegisterResponse},
    types::User,
};

use crate::{
    auth::{self, Auth},
    query::SelectOne,
    result::ApiResult,
    state::AppState,
};

pub async fn get_profile(State(state): State<AppState>, Auth(user_id): Auth) -> ApiResult<User> {
    Ok(Json(
        User::select_one(user_id).fetch_one(&state.pool).await?,
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

    let user_id: Id<User> = sqlx::query_scalar!(
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
    .await?
    .into();

    Ok(Json(RegisterResponse {
        api_key: auth::api_key(&state, user_id),
    }))
}
