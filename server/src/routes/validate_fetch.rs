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
    result::{AppError, AppResult, ResultExt},
    state::AppState,
};

pub async fn validate_fetch(
    State(state): State<AppState>,
    Path(project_id): Path<Id<Project>>,
) -> AppResult<Json<Vec<Task>>, ValidateFetchError> {
    let mut tx = state.pool.begin().await?;
    let project_result = sqlx::query_as_unchecked!(
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
    .fetch_optional(&mut *tx)
    .await?;

    match project_result{
        Some(project) => {
             let task = sqlx::query_as_unchecked!(
                    Task,
                    r#"
                    SELECT
                        t.*
                    FROM
                        tasks t
                        JOIN assignments a ON
                            a.task_id = t.id
                    WHERE
                        a.state not in ('canceled', 'expired')
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
                tx.commit().await?;
                Ok(Json(task.into_iter().collect()))
        },
        None => Err(AppError::Specific(ValidateFetchError::InvalidProject))
    }

   
}
