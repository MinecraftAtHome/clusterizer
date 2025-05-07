use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::types::{Project, Result};

use crate::{result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Project>> {
    Ok(Json(
        sqlx::query_as!(Project, "SELECT * FROM projects")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> ApiResult<Project> {
    Ok(Json(
        sqlx::query_as!(Project, "SELECT * FROM projects WHERE id = $1", project_id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn results(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> ApiResult<Vec<Result>> {
    Ok(Json(
        sqlx::query_as!(
            Result,
            "
            SELECT
                r.*
            FROM
                tasks t
                JOIN assignments a
                    ON a.task_id = t.id
                JOIN results r
                    ON r.assignment_id = a.id
            WHERE
                t.project_id = $1
            ",
            project_id
        )
        .fetch_all(&state.pool)
        .await?,
    ))
}

pub async fn versions(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Vec<ProjectVersion>> {
    Ok(Json(
        sqlx::query_as!(ProjectVersion, 
            
            "SELECT * FROM project_versions WHERE project_id = $1", id)
            .fetch_all(&state.pool)
            .await?,
    ))
}