use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::types::{Project, ProjectVersion, Result};

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

pub async fn get_results(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> ApiResult<Vec<Result>> {
    let results = sqlx::query_as!(
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
    .await?;

    if results.is_empty() {
        sqlx::query_scalar!("SELECT 1 FROM projects WHERE id = $1", project_id)
            .fetch_one(&state.pool)
            .await?;
    }

    Ok(Json(results))
}

pub async fn get_project_version(
    State(state): State<AppState>,
    Path((project_id, platform_id)): Path<(i64, i64)>,
) -> ApiResult<ProjectVersion> {
    Ok(Json(
        sqlx::query_as!(
            ProjectVersion,
            "SELECT * FROM project_versions WHERE project_id = $1 AND platform_id = $2",
            project_id,
            platform_id,
        )
        .fetch_one(&state.pool)
        .await?,
    ))
}
