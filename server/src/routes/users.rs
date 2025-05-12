use axum::{Json, extract::State};
use clusterizer_common::{
    errors::Infallible,
    id::Id,
    messages::{RegisterRequest, RegisterResponse},
    types::User,
};

use crate::{
    auth::{self, Auth},
    query::SelectOne,
    result::AppResult,
    state::AppState,
};

pub async fn get_profile(
    State(state): State<AppState>,
    Auth(user_id): Auth,
) -> AppResult<User, Infallible> {
    Ok(Json(
        User::select_one(user_id).fetch_one(&state.pool).await?,
    ))
}

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> AppResult<RegisterResponse, Infallible> {
    if request.name.len() < 3 {
        Err(todo!() as Infallible)?;
    }

    if request.name.len() > 32 {
        Err(todo!() as Infallible)?;
    }

    if !request
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        Err(todo!() as Infallible)?;
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
