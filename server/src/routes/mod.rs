use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::id::Id;

use crate::{
    query::{SelectAll, SelectAllBy, SelectOne, SelectOneBy},
    result::ApiResult,
    state::AppState,
};

pub mod tasks;
pub mod users;

pub async fn get_all<T: SelectAll + Send + Unpin>(
    State(state): State<AppState>,
) -> ApiResult<Vec<T>> {
    Ok(Json(T::select_all().fetch_all(&state.pool).await?))
}

pub async fn get_all_by<T: SelectAllBy<U> + Send + Unpin, U: SelectOne + Send + Unpin>(
    State(state): State<AppState>,
    Path(id): Path<Id<U>>,
) -> ApiResult<Vec<T>> {
    let result = T::select_all_by(id).fetch_all(&state.pool).await?;

    if result.is_empty() {
        U::select_one(id).fetch_one(&state.pool).await?;
    }

    Ok(Json(result))
}

pub async fn get_one<T: SelectOne + Send + Unpin>(
    State(state): State<AppState>,
    Path(id): Path<Id<T>>,
) -> ApiResult<T> {
    Ok(Json(T::select_one(id).fetch_one(&state.pool).await?))
}

pub async fn get_one_by<T: SelectOneBy<U> + Send + Unpin, U: SelectOne + Send + Unpin>(
    State(state): State<AppState>,
    Path(id): Path<Id<U>>,
) -> ApiResult<Option<T>> {
    let result = T::select_one_by(id).fetch_optional(&state.pool).await?;

    if result.is_none() {
        U::select_one(id).fetch_one(&state.pool).await?;
    }

    Ok(Json(result))
}
