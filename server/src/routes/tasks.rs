use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::types::Task;

use crate::{result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Task>> {
    Ok(Json(
        sqlx::query_as!(Task, "SELECT * FROM tasks")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Task> {
    Ok(Json(
        sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", id)
            .fetch_one(&state.pool)
            .await?,
    ))
}
