use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::types::Task;

use crate::{auth::Auth, result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Task>> {
    Ok(Json(
        sqlx::query_as!(Task, "SELECT * FROM tasks")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<Task> {
    Ok(Json(
        sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn fetch(State(state): State<AppState>, Auth(user_id): Auth) -> ApiResult<Vec<Task>> {
    let _guard = state.fetch_mutex.lock().await;

    let task = sqlx::query_as!(
        Task,
        "
        SELECT
            t.*
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

    if let Some(task) = task {
        sqlx::query!(
            "
            INSERT INTO assignments (
                task_id,
                user_id
            ) VALUES (
                $1,
                $2
            )
            ",
            task.id,
            user_id
        )
        .execute(&state.pool)
        .await?;

        Ok(Json(vec![task]))
    } else {
        Ok(Json(vec![]))
    }
}
