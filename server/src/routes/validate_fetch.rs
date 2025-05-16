use axum::{Json, extract::State};
use clusterizer_common::{errors::ValidateFetchError, requests::FetchTasksRequest, types::Task};

use crate::{result::AppResult, state::AppState};

pub async fn validate_fetch(
    State(state): State<AppState>,
    Json(request): Json<FetchTasksRequest>,
) -> AppResult<Json<Vec<Task>>, ValidateFetchError> {
    let count = sqlx::query_scalar_unchecked!(
        r#"
        SELECT
            count(*) as "count!"
        FROM
            projects
        WHERE
            id = ANY($1)
        "#,
        request.project_ids
    )
    .fetch_one(&state.pool)
    .await? as usize;

    if count != request.project_ids.len() {
        Err(ValidateFetchError::InvalidProject)?;
    }

    let task = sqlx::query_as_unchecked!(
        Task,
        r#"
        SELECT
            t.*
        FROM
            tasks t
            JOIN assignments a ON
                a.task_id = t.id
            JOIN results r ON
                r.assignment_id = a.id
        GROUP BY
            t.id
        HAVING
            t.project_id = ANY($1)
            AND count(r.id) >= t.assignments_needed
        "#,
        request.project_ids,
    )
    .fetch_optional(&state.pool)
    .await?;

    Ok(Json(task.into_iter().collect()))
}
