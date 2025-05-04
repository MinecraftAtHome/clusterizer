use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::types::Project;

use crate::{result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Project>> {
    Ok(Json(
        sqlx::query_as!(Project, "SELECT * FROM projects")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Project> {
    Ok(Json(
        sqlx::query_as!(Project, "SELECT * FROM projects WHERE id = $1", id)
            .fetch_one(&state.pool)
            .await?,
    ))
}
