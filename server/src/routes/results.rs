use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{messages::SubmitRequest, types::Result};

use crate::{auth::Auth, result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Result>> {
    Ok(Json(
        sqlx::query_as!(Result, "SELECT * FROM results")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Result> {
    Ok(Json(
        sqlx::query_as!(Result, "SELECT * FROM results WHERE id = $1", id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn submit(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(request): Json<SubmitRequest>,
) -> ApiResult<()> {
    let record = sqlx::query!(
        "
        SELECT
            id
        FROM
            assignments
        WHERE
            task_id = $1 AND
            user_id = $2 AND
            NOT canceled
        ",
        request.task_id,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    sqlx::query!(
        "
        INSERT INTO results (
            assignment_id,
            stdout,
            stderr,
            exit_code
        ) VALUES (
            $1,
            $2,
            $3,
            $4
        )
        ",
        record.id,
        request.stdout,
        request.stderr,
        request.exit_code
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(()))
}
