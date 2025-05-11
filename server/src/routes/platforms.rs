use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{id::Id, types::Platform};

use crate::{
    query::{QueryAll, QueryOne},
    result::ApiResult,
    state::AppState,
};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Platform>> {
    Ok(Json(Platform::query_all().fetch_all(&state.pool).await?))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(platform_id): Path<Id<Platform>>,
) -> ApiResult<Platform> {
    Ok(Json(
        Platform::query_one(platform_id)
            .fetch_one(&state.pool)
            .await?,
    ))
}
