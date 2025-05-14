use axum::{Json, extract::State};
use clusterizer_common::{
    errors::RegisterError, requests::RegisterRequest, responses::RegisterResponse,
};

use crate::{auth, result::AppResult, state::AppState};

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> AppResult<Json<RegisterResponse>, RegisterError> {
    if request.name.len() < 3 {
        Err(RegisterError::TooShort)?;
    }

    if request.name.len() > 32 {
        Err(RegisterError::TooLong)?;
    }

    if !request
        .name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        Err(RegisterError::InvalidCharacter)?;
    }

    let user_id = sqlx::query_scalar_unchecked!(
        r#"
        INSERT INTO users (
            name
        ) VALUES (
            $1
        )
        RETURNING id "id: _"
        "#,
        request.name
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(RegisterResponse {
        api_key: auth::api_key(&state, user_id),
    }))
}
