use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::types::ProjectVersion;

use crate::{result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<ProjectVersion>> {
    Ok(Json(
        sqlx::query_as!(ProjectVersion, "SELECT * FROM project_versions")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<ProjectVersion> {
    Ok(Json(
        sqlx::query_as!(
            ProjectVersion,
            "SELECT * FROM project_versions WHERE id = $1",
            id
        )
        .fetch_one(&state.pool)
        .await?,
    ))
}
