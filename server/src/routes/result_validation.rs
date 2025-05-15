use axum::{extract::State, Json};
use clusterizer_common::{errors::{ValidateFetchError, ValidateOkError}, requests::{validate_ok_request::ValidateOkRequest, CanonicalResultRequest, FetchTasksRequest}, types::Task};

use crate::{auth::Auth, result::AppResult, state::AppState};


pub async fn validate_fetch(
    State(state): State<AppState>,
    Auth(user_id): Auth,
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
Auth(user_id): Auth,
Json(request): Json<ValidateOkRequest>,) -> AppResult<(), ValidateOkError> {
    let task_count = sqlx::query_scalar_unchecked!(
        r#"
        SELECT
            count(1) as "count!"
        FROM
            tasks
        WHERE
            id = $1
            AND canonical_result_id IS NULL
        "#,
        request.task_id.raw()
    )
    .fetch_one(&state.pool)
    .await? as usize;

    if task_count != 1 {
        Err(ValidateOkError::InvalidTask)?;
    }

    sqlx::query_unchecked!(
        r#"
        UPDATE tasks
        SET canonical_result_id = $2
        WHERE
            id = $1
        "#,
        request.task_id.raw(),
        request.canonical_result_id.raw()
    )
    .execute(&state.pool)
    .await?;
    sqlx::query_unchecked!(
        r#"
        UPDATE result
        SET validated = True
        WHERE
            id = ANY($1)
        "#,
        request.result_ids
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}



pub async fn validate_err(
    State(state): State<AppState>,
Auth(user_id): Auth,
Json(request): Json<ValidateErrRequest>,) -> AppResult<(), ValidateErrError> {
    let project = Project::get_one_by(request.task_id)
    let result_count = sqlx::query_scalar_unchecked!(
        r#"
        SELECT
            count(1) as "count!"
        FROM
            result r
        JOIN assignments a ON
            a.id = r.assignment_id
        JOIN tasks t ON
            t.id = a.task_id
        WHERE
            r.id = $1
        "#,
        request.task_id.raw()
    )
    .fetch_one(&state.pool)
    .await? as usize;

    let task_count = sqlx::query_scalar_unchecked!(
        r#"
        SELECT
            count(1) as "count!"
        FROM
            tasks
        WHERE
            id = $1
            AND canonical_result_id IS NULL
        "#,
        request.task_id.raw()
    )
    .fetch_one(&state.pool)
    .await? as usize;

    if task_count != 1 {
        Err(ValidateErrError::InvalidTask)?;
    }
    if request.assignments_needed == 0 || request.assignments_needed > {

    }

    sqlx::query_unchecked!(
        r#"
        UPDATE tasks
        SET canonical_result_id = $2
        WHERE
            id = $1
        "#,
        request.task_id.raw(),
        request.canonical_result_id.raw()
    )
    .execute(&state.pool)
    .await?;
    sqlx::query_unchecked!(
        r#"
        UPDATE result
        SET validated = True
        WHERE
            id = ANY($1)
        "#,
        request.result_ids
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}


