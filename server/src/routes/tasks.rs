use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{messages::SubmitRequest, types::Task};

use crate::{auth::Auth, result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Task>> {
    Ok(Json(
        sqlx::query_as!(Task, "SELECT * FROM tasks")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(State(state): State<AppState>, Path(task_id): Path<i64>) -> ApiResult<Task> {
    Ok(Json(
        sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1", task_id)
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
                AND a.canceled_at IS NULL
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

pub async fn submit(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    Auth(user_id): Auth,
    Json(request): Json<SubmitRequest>,
) -> ApiResult<()> {
    let assignment = sqlx::query!(
        "
        SELECT
            id
        FROM
            assignments
        WHERE
            task_id = $1 AND
            user_id = $2 AND
            canceled_at IS NULL
        ",
        task_id,
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
        assignment.id,
        request.stdout,
        request.stderr,
        request.exit_code
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(()))
}
