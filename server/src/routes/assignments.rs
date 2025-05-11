use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{id::Id, types::Assignment};

use crate::{
    query::{QueryAll, QueryOne},
    result::ApiResult,
    state::AppState,
};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Assignment>> {
    Ok(Json(Assignment::query_all().fetch_all(&state.pool).await?))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(assignment_id): Path<Id<Assignment>>,
) -> ApiResult<Assignment> {
    Ok(Json(
        Assignment::query_one(assignment_id)
            .fetch_one(&state.pool)
            .await?,
    ))
}
