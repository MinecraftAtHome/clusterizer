use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{id::Id, types::ProjectVersion};

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
    Path(project_version_id): Path<Id<ProjectVersion>>,
) -> ApiResult<ProjectVersion> {
    Ok(Json(
        sqlx::query_as!(
            ProjectVersion,
            "SELECT * FROM project_versions WHERE id = $1",
            project_version_id.raw()
        )
        .fetch_one(&state.pool)
        .await?,
    ))
}
