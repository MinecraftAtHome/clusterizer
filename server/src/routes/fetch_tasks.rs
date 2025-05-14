use axum::{Json, extract::State};
use clusterizer_common::{errors::FetchTasksError, requests::FetchTasksRequest, types::Task};

use crate::{auth::Auth, result::AppResult, state::AppState};

pub async fn fetch_tasks(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(request): Json<FetchTasksRequest>,
) -> AppResult<Json<Vec<Task>>, FetchTasksError> {
    let mut transaction = state
        .pool
        .begin_with("BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .await?;

    let count = sqlx::query_scalar_unchecked!(
        r#"
        SELECT
            count(*) as "count!"
        FROM
            projects
        WHERE
            id = ANY($1)
            AND active
        "#,
        request.project_ids
    )
    .fetch_one(&mut *transaction)
    .await? as usize;

    if count != request.project_ids.len() {
        Err(FetchTasksError::InvalidProject)?;
    }

    let task = sqlx::query_as_unchecked!(
        Task,
        r#"
        SELECT
            t.*
        FROM
            tasks t
            LEFT JOIN assignments a
                ON a.task_id = t.id
                AND a.canceled_at IS NULL
                AND a.user_id = $1
        WHERE
            t.assignments_remaining > 0
            AND t.project_id = ANY($2)
            AND a.id IS NULL
        "#,
        user_id,
        request.project_ids
    )
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(task) = task {
        sqlx::query_unchecked!(
            r#"
            UPDATE
                tasks
            SET
                assignments_remaining = assignments_remaining - 1
            WHERE
                id = $1
            "#,
            task.id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query_unchecked!(
            r#"
            INSERT INTO assignments (
                task_id,
                user_id
            ) VALUES (
                $1,
                $2
            )
            "#,
            task.id,
            user_id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(Json(vec![task]))
    } else {
        transaction.rollback().await?;

        Ok(Json(vec![]))
    }
}
