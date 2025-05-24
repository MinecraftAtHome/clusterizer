use axum::{Json, extract::State};
use clusterizer_common::{
    errors::FetchTasksError,
    records::{Project, Task},
    requests::FetchTasksRequest,
};

use crate::{
    auth::Auth,
    result::{AppError, AppResult},
    state::AppState,
};

pub async fn fetch_tasks(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(request): Json<FetchTasksRequest>,
) -> AppResult<Json<Vec<Task>>, FetchTasksError> {
    let mut tx = state.pool.begin().await?;

    let projects = sqlx::query_as_unchecked!(
        Project,
        r#"
        SELECT
            *
        FROM
            projects
        WHERE
            id = ANY($1)
        "#,
        request.project_ids
    )
    .fetch_all(&mut *tx)
    .await?;

    if projects.len() != request.project_ids.len() {
        Err(AppError::Specific(FetchTasksError::InvalidProject))?;
    }

    let project_ids: Vec<_> = projects
        .into_iter()
        .filter(|project| project.disabled_at.is_none())
        .map(|project| project.id)
        .collect();

    let tasks = sqlx::query_as_unchecked!(
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
        LIMIT $3
        "#,
        project_ids,
        user_id,
        request.limit.min(32) as i64,
    )
    .fetch_all(&mut *tx)
    .await?;

    for task in &tasks {
        sqlx::query_unchecked!(
            r#"
            INSERT INTO assignments (
                task_id,
                user_id,
                deadline_at
            ) VALUES (
                $1,
                $2,
                now() + $3
            )
            "#,
            task.id,
            user_id,
            task.deadline,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(tasks))
}
