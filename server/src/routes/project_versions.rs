use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{id::Id, types::ProjectVersion};

use crate::{
    query::{QueryAll, QueryOne},
    result::ApiResult,
    state::AppState,
};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<ProjectVersion>> {
    Ok(Json(
        ProjectVersion::query_all().fetch_all(&state.pool).await?,
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(project_version_id): Path<Id<ProjectVersion>>,
) -> ApiResult<ProjectVersion> {
    Ok(Json(
        ProjectVersion::query_one(project_version_id)
            .fetch_one(&state.pool)
            .await?,
    ))
}
