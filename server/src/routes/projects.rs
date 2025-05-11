use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    id::Id,
    types::{Platform, Project, ProjectVersion, Result},
};

use crate::{
    query::{QueryAll, QueryOne},
    result::ApiResult,
    state::AppState,
};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Project>> {
    Ok(Json(Project::query_all().fetch_all(&state.pool).await?))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(project_id): Path<Id<Project>>,
) -> ApiResult<Project> {
    Ok(Json(
        Project::query_one(project_id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn get_results(
    State(state): State<AppState>,
    Path(project_id): Path<Id<Project>>,
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
        project_id.raw()
    )
    .fetch_all(&state.pool)
    .await?;

    if results.is_empty() {
        sqlx::query_scalar!("SELECT 1 FROM projects WHERE id = $1", project_id.raw())
            .fetch_one(&state.pool)
            .await?;
    }

    Ok(Json(results))
}

pub async fn get_project_version(
    State(state): State<AppState>,
    Path((project_id, platform_id)): Path<(Id<Project>, Id<Platform>)>,
) -> ApiResult<ProjectVersion> {
    Ok(Json(
        sqlx::query_as!(
            ProjectVersion,
            "SELECT * FROM project_versions WHERE project_id = $1 AND platform_id = $2",
            project_id.raw(),
            platform_id.raw(),
        )
        .fetch_one(&state.pool)
        .await?,
    ))
}
