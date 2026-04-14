use axum::{Json, extract::State};
use clusterizer_common::{
    errors::CreateFileError,
    records::{File, FileBuilder, Insert, Select},
    requests::CreateFileRequest,
    types::Id,
};
use url::Url;

use crate::{
    auth::Auth,
    result::{AppError, AppResult},
    state::AppState,
};

pub async fn create_file(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(request): Json<CreateFileRequest>,
) -> AppResult<Json<Id<File>>, CreateFileError> {
    let user = user_id.select().fetch_one(&state.pool).await?;

    if !user.is_admin {
        Err(AppError::Specific(CreateFileError::Forbidden))?;
    }

    Url::parse(&request.url).map_err(|_| AppError::Specific(CreateFileError::InvalidUrl))?;

    let file_id = FileBuilder {
        url: request.url,
        hash: request.hash,
    }
    .insert()
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(file_id))
}
