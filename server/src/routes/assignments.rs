use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::types::Assignment;

use crate::{auth::Auth, result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Assignment>> {
    Ok(Json(
        sqlx::query_as!(Assignment, "SELECT * FROM assignments")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Assignment> {
    Ok(Json(
        sqlx::query_as!(Assignment, "SELECT * FROM assignments WHERE id = $1", id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn fetch(
    State(state): State<AppState>,
    Auth(user_id): Auth,
) -> ApiResult<Vec<Assignment>> {
    let _guard = state.fetch_mutex.lock().await;

    let record = sqlx::query!(
        "
        SELECT
            t.id
        FROM
            tasks t
            JOIN projects p
                ON t.project_id = p.id
                AND p.active
            LEFT JOIN assignments a
                ON a.task_id = t.id
                AND NOT a.canceled
        WHERE
            a.id IS NULL
        "
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(record) = record {
        let assignment = sqlx::query_as!(
            Assignment,
            "
            INSERT INTO assignments (
                task_id,
                user_id
            ) VALUES (
                $1,
                $2
            )
            RETURNING *
            ",
            record.id,
            user_id
        )
        .fetch_one(&state.pool)
        .await?;

        Ok(Json(vec![assignment]))
    } else {
        Ok(Json(vec![]))
    }
}
