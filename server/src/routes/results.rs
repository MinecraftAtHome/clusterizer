use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{id::Id, types::Result};

use crate::{
    query::{QueryAll, QueryOne},
    result::ApiResult,
    state::AppState,
};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Result>> {
    Ok(Json(Result::query_all().fetch_all(&state.pool).await?))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(result_id): Path<Id<Result>>,
) -> ApiResult<Result> {
    Ok(Json(
        Result::query_one(result_id).fetch_one(&state.pool).await?,
    ))
}
