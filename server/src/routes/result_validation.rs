use axum::{Json, extract::State};
use clusterizer_common::{
    errors::{ValidateErrError, ValidateFetchError, ValidateOkError},
    requests::{FetchTasksRequest, ValidateErrRequest, validate_ok_request::ValidateOkRequest},
    types::{Project, Task},
};

use crate::{query::SelectOne, result::AppResult, state::AppState};

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
            AND active
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

pub async fn validate_ok(
    State(state): State<AppState>,
    Json(request): Json<ValidateOkRequest>,
) -> AppResult<(), ValidateOkError> {
    match Task::select_one(request.task_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| ValidateOkError::InvalidTask)?
    {
        Some(_) => {
            sqlx::query_unchecked!(
                r#"
                UPDATE tasks
                SET canonical_result_id =  
                                            (SELECT 
                                                id 
                                            FROM 
                                                results 
                                            WHERE 
                                                id = ANY($2) 
                                            ORDER BY 
                                                created_at DESC 
                                            LIMIT 1
                                            )
                WHERE
                    id = $1
                "#,
                request.task_id.raw(),
                request.result_ids
            )
            .execute(&state.pool)
            .await?;
            sqlx::query_unchecked!(
                r#"
                UPDATE results
                SET is_validated = True
                WHERE
                    id = ANY($1)
                "#,
                request.result_ids
            )
            .execute(&state.pool)
            .await?;
        }
        None => Err(ValidateOkError::InvalidTask)?,
    }

    Ok(())
}

pub async fn validate_err(
    State(state): State<AppState>,
    Json(request): Json<ValidateErrRequest>,
) -> AppResult<(), ValidateErrError> {
    match Task::select_one(request.task_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| ValidateErrError::InvalidTask)?
    {
        Some(task) => {
            let project = Project::select_one(task.project_id)
                .fetch_one(&state.pool)
                .await
                .map_err(|_| ValidateErrError::InvalidTask)?;
            if request.assignments_needed == 0
                || request.assignments_needed > project.quorum + task.assignments_needed
            {
                Err(ValidateErrError::AssignmentsNeededOutOfBounds)?
            } else {
                sqlx::query_unchecked!(
                    r#"
                    UPDATE tasks
                    SET assignments_needed = assignments_needed + $2
                    WHERE
                        id = $1
                    "#,
                    request.task_id.raw(),
                    request.assignments_needed
                )
                .execute(&state.pool)
                .await?;
            }
        }
        None => Err(ValidateErrError::InvalidTask)?,
    }
    Ok(())
}
