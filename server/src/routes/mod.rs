use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    errors::{Infallible, NotFound},
    types::Id,
};

use crate::{
    query::{SelectAll, SelectAllBy, SelectOne, SelectOneBy},
    result::{AppResult, ResultExt},
    state::AppState,
};

pub mod fetch_tasks;
pub mod register;
pub mod submit_result;

pub async fn get_all<T: SelectAll + Send + Unpin>(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<T>>, Infallible> {
    Ok(Json(T::select_all().fetch_all(&state.pool).await?))
}

pub async fn get_all_by<T: SelectAllBy<U> + Send + Unpin, U: SelectOne + Send + Unpin>(
    State(state): State<AppState>,
    Path(id): Path<Id<U>>,
) -> AppResult<Json<Vec<T>>, NotFound> {
    let result = T::select_all_by(id).fetch_all(&state.pool).await?;

    if result.is_empty() {
        U::select_one(id)
            .fetch_one(&state.pool)
            .await
            .map_not_found(NotFound)?;
    }

    Ok(Json(result))
}

pub async fn get_one<T: SelectOne + Send + Unpin>(
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

pub async fn get_one_by<T: SelectOneBy<U> + Send + Unpin, U: SelectOne + Send + Unpin>(
    State(state): State<AppState>,
    Path(id): Path<Id<U>>,
) -> AppResult<Json<Option<T>>, NotFound> {
    let result = T::select_one_by(id).fetch_optional(&state.pool).await?;

    if result.is_none() {
        U::select_one(id)
            .fetch_one(&state.pool)
            .await
            .map_not_found(NotFound)?;
    }

    Ok(Json(result))
}
