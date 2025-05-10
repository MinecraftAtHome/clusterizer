use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{id::Id, types::Platform};

use crate::{result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Platform>> {
    Ok(Json(
        sqlx::query_as!(Platform, "SELECT * FROM platforms")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(platform_id): Path<Id<Platform>>,
) -> ApiResult<Platform> {
    Ok(Json(
        sqlx::query_as!(
            Platform,
            "SELECT * FROM platforms WHERE id = $1",
            platform_id.raw()
        )
        .fetch_one(&state.pool)
        .await?,
    ))
}
