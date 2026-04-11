use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    errors::{Infallible, NotFound},
    records::{Record, Select},
    types::Id,
};
use serde_qs::web::QsQuery;

use crate::{
    result::{AppResult, ResultExt},
    state::AppState,
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

pub async fn get_all<T: Record + Send + Unpin>(
    State(state): State<AppState>,
    QsQuery(filter): QsQuery<T::Filter>,
) -> AppResult<Json<Vec<T>>, Infallible>
where
    T::Filter: Select<Record = T>,
{
    Ok(Json(filter.select().fetch_all(&state.pool).await?))
}

pub async fn get_one<T: Record + Send + Unpin>(
    State(state): State<AppState>,
    Path(id): Path<Id<T>>,
) -> AppResult<Json<T>, NotFound>
where
    Id<T>: Select<Record = T>,
{
    Ok(Json(
        id.select()
            .fetch_one(&state.pool)
            .await
            .map_not_found(NotFound)?,
    ))
}
