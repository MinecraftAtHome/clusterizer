use axum::{Json, extract::State};
use clusterizer_common::{errors::FetchTasksError, requests::FetchTasksRequest, types::Task};

use crate::{auth::Auth, result::AppResult, state::AppState};

pub async fn fetch_tasks(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(request): Json<FetchTasksRequest>,
) -> AppResult<Json<Vec<Task>>, FetchTasksError> {
    let mut tx = state.pool.begin().await?;

    let count = sqlx::query_scalar_unchecked!(
        r#"
        SELECT
            count(*) as "count!"
        FROM
            projects
        WHERE
            id = ANY($1)
            AND disabled_at IS NULL
        "#,
        request.project_ids
    )
    .fetch_one(&mut *tx)
    .await? as usize;

    if count != request.project_ids.len() {
        Err(FetchTasksError::InvalidProject)?;
    }

    let task = sqlx::query_as_unchecked!(
        Task,
        r#"
        SELECT
            *
        FROM
            tasks
        WHERE
            project_id = ANY($1)
            AND cardinality(assignment_user_ids) < assignments_needed
            AND $2 != ALL(assignment_user_ids)
        FOR UPDATE SKIP LOCKED
        LIMIT 1
        "#,
        request.project_ids,
        user_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(task) = &task {
        sqlx::query_unchecked!(
            r#"
            UPDATE
                tasks
            SET
                assignment_user_ids = assignment_user_ids || $2
            WHERE
                id = $1
            "#,
            task.id,
            user_id,
        )
        .execute(&mut *tx)
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
            user_id,
        )
        .execute(&mut *tx)
        .await?;
    };

    tx.commit().await?;

    Ok(Json(task.into_iter().collect()))
}
