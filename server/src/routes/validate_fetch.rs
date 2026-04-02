use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    errors::ValidateFetchError,
    records::{Project, Task},
    types::Id,
};

use crate::{
    auth::Auth,
    result::{AppError, AppResult, ResultExt},
    state::AppState,
};

pub async fn validate_fetch(
    State(state): State<AppState>,
    Path(project_id): Path<Id<Project>>,
    Auth(user_id): Auth,
) -> AppResult<Json<Vec<Task>>, ValidateFetchError> {
    let project = sqlx::query_as_unchecked!(
        Project,
        r#"
        SELECT
            *
        FROM
            projects
        WHERE
            id = $1
        "#,
        project_id,
    )
    .fetch_one(&state.pool)
    .await
    .map_not_found(ValidateFetchError::InvalidProject)?;

    if project.created_by_user_id != user_id {
        Err(AppError::Specific(ValidateFetchError::Forbidden))?;
    }

    let tasks = sqlx::query_as_unchecked!(
        Task,
        r#"
        SELECT
            t.*
        FROM
            tasks t
            JOIN assignments a ON
                a.task_id = t.id
            LEFT JOIN results r ON
                r.assignment_id = a.id
                AND r.state = 'init'
        WHERE
            a.state = 'submitted'
        GROUP BY
            t.id
        HAVING
            t.project_id = $1
            AND count(a.id) >= t.assignments_needed
            AND count(r.id) > 0
        "#,
        project.id,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(tasks))
}
