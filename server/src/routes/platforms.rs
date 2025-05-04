use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::types::Platform;

use crate::{result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Platform>> {
    Ok(Json(
        sqlx::query_as!(Platform, "SELECT * FROM platforms")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Platform> {
    Ok(Json(
        sqlx::query_as!(Platform, "SELECT * FROM platforms WHERE id = $1", id)
            .fetch_one(&state.pool)
            .await?,
    ))
}
