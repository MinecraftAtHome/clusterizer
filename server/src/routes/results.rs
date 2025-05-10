use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{id::Id, types::Result};

use crate::{result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Result>> {
    Ok(Json(
        sqlx::query_as!(Result, "SELECT * FROM results")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(result_id): Path<Id<Result>>,
) -> ApiResult<Result> {
    Ok(Json(
        sqlx::query_as!(
            Result,
            "SELECT * FROM results WHERE id = $1",
            result_id.raw()
        )
        .fetch_one(&state.pool)
        .await?,
    ))
}
