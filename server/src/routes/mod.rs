use axum::{
    Json,
    extract::{Path, Query, State},
};
use clusterizer_common::{
    errors::{Infallible, NotFound},
    types::Id,
};

use crate::{
    result::{AppResult, ResultExt},
    state::AppState,
    util::Select,
};

pub mod fetch_tasks;
pub mod register;
pub mod submit_result;
pub mod validate_fetch;
pub mod validate_submit;

pub use fetch_tasks::fetch_tasks;
pub use register::register;
pub use submit_result::submit_result;
pub use validate_fetch::validate_fetch;
pub use validate_submit::validate_submit;

pub async fn get_all<T: Select + Send + Unpin>(
    State(state): State<AppState>,
    Query(filter): Query<T::Filter>,
) -> AppResult<Json<Vec<T>>, Infallible> {
    Ok(Json(T::select_all(&filter).fetch_all(&state.pool).await?))
}

pub async fn get_one<T: Select + Send + Unpin>(
    State(state): State<AppState>,
    Path(id): Path<Id<T>>,
) -> AppResult<Json<T>, NotFound> {
    Ok(Json(
        T::select_one(id)
            .fetch_one(&state.pool)
            .await
            .map_not_found(NotFound)?,
    ))
}
