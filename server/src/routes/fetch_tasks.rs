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
        project_ids,
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
