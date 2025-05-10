use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{id::Id, types::Assignment};

use crate::{result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Assignment>> {
    Ok(Json(
        sqlx::query_as!(Assignment, "SELECT * FROM assignments")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(assignment_id): Path<Id<Assignment>>,
) -> ApiResult<Assignment> {
    Ok(Json(
        sqlx::query_as!(
            Assignment,
            "SELECT * FROM assignments WHERE id = $1",
            assignment_id.raw()
        )
        .fetch_one(&state.pool)
        .await?,
    ))
}
