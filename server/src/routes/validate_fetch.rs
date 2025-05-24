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
    result::{AppResult, ResultExt},
    state::AppState,
};

pub async fn validate_fetch(
    State(state): State<AppState>,
    Path(project_id): Path<Id<Project>>,
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
        project_id
    )
    .fetch_one(&state.pool)
    .await
    .map_not_found(ValidateFetchError::InvalidProject)?;

    let tasks = sqlx::query_as_unchecked!(
        Task,
        r#"
            SELECT
                t.*
            FROM
                tasks t
                JOIN assignments a ON
                    a.task_id = t.id
            WHERE
                a.state not in ('canceled', 'init', 'expired')
            GROUP BY
                t.id
            HAVING
                t.project_id = $1
                AND (
                    count(a.id) >= t.assignments_needed
                    OR t.canonical_result_id IS NOT NULL
                )
            "#,
        project.id
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(tasks))
}
