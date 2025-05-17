use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    errors::ValidateErrError,
    id::Id,
    requests::ValidateErrRequest,
    types::{Assignment, AssignmentState, Project, Task},
};

use crate::{
    query::{SelectAllBy, SelectOne},
    result::AppResult,
    state::AppState,
    util::set_assignment_state,
};

pub async fn validate_err(
    State(state): State<AppState>,
    Json(request): Json<ValidateErrRequest>,
) -> AppResult<(), ValidateErrError> {

    let assignments_error = sqlx::query_as_unchecked!(
        Assignment,
        r#"
            SELECT
                *
            FROM
                assignments
            WHERE
                id = ANY($1)
        "#,
        request.assignments_error
    )
    .fetch_all(&state.pool)
    .await?;


    let assignments_inconclusive = sqlx::query_as_unchecked!(
        Assignment,
        r#"
            SELECT
                *
            FROM
                assignments
            WHERE
                id = ANY($1)
        "#,
        request.assignments_inconclusive
    )
    .fetch_all(&state.pool)
    .await?;

    if assignments_inconclusive.len() != request.assignments_inconclusive.len() || assignments_error.len() != request.assignments_error.len() {
        Err(ValidateErrError::InvalidAssignment)?;
    }

    let mut task_error_ids = Vec::from_iter(assignments_error.into_iter().map(|assignment| assignment.task_id));
    task_error_ids.sort();
    task_error_ids.dedup();

    let mut task_inconclusive_ids = Vec::from_iter(assignments_inconclusive.into_iter().map(|assignment| assignment.task_id));
    task_inconclusive_ids.sort();
    task_inconclusive_ids.dedup();

    if task_error_ids.len() > 1 || task_inconclusive_ids.len() > 1 {
        Err(ValidateErrError::AssignmentTaskRelationshipError)?
    }
    
    if task_error_ids[0] != task_inconclusive_ids[0]{
        Err(ValidateErrError::RequestAssignmentsRelationshipError)?
    }

    let task_error = Task::select_one(task_error_ids[0]).fetch_one(&state.pool).await?;
    let task_inconclusive = Task::select_one(task_inconclusive_ids[0]).fetch_one(&state.pool).await?;

    if task_error.canonical_result_id.is_some() || task_inconclusive.canonical_result_id.is_some() {
        Err(ValidateErrError::CanonicalResultExists)?
    }

    let project_inconclusive = Project::select_one(task_inconclusive.project_id)
        .fetch_one(&state.pool)
        .await?;

    let result_error_count = sqlx::query_scalar_unchecked!(
        r#"
            SELECT
                count(1) as "count!"
            FROM
                results r
            JOIN assignments a
                ON a.id = r.assignment_id
            JOIN tasks t
                ON t.id = a.task_id
            WHERE
                t.id = $1
        "#,
        task_error.id
    )
    .fetch_one(&state.pool)
    .await? as i32;

    let result_inconclusive_count = sqlx::query_scalar_unchecked!(
        r#"
            SELECT
                count(1) as "count!"
            FROM
                results r
            JOIN assignments a
                ON a.id = r.assignment_id
            JOIN tasks t
                ON t.id = a.task_id
            WHERE
                t.id = $1
        "#,
        task_inconclusive.id
    )
    .fetch_one(&state.pool)
    .await? as i32;

    if request.assignments_needed <= 0
        || request.assignments_needed + task_inconclusive.assignments_needed > project_inconclusive.quorum + result_inconclusive_count + result_error_count
    {
        Err(ValidateErrError::AssignmentsNeededOutOfBounds)?
    } else {
        //Set assignments needed
        sqlx::query_unchecked!(
            r#"
            UPDATE tasks
            SET assignments_needed = assignments_needed + $2
            WHERE
                id = $1
            "#,
            task_inconclusive.id,
            request.assignments_needed
        )
        .execute(&state.pool)
        .await?;

        //Set inconclusive validate state
        set_assignment_state::set_assignment_state(
            &state,
            AssignmentState::Inconclusive,
            &request.assignments_inconclusive,
        )
        .await?;

        //Set invalid validate state
        set_assignment_state::set_assignment_state(
            &state,
            AssignmentState::Invalid,
            &request.assignments_error,
        )
        .await?;
    }

    Ok(())
}
