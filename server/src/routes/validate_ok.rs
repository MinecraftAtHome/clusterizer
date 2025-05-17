use axum::{Json, extract::State};
use clusterizer_common::{
    errors::ValidateOkError,
    records::{Assignment, Project, Task},
    requests::ValidateOkRequest,
    types::AssignmentState,
};

use crate::{query::SelectOne, result::AppResult, state::AppState, util::set_assignment_state};

pub async fn validate_ok(
    State(state): State<AppState>,
    Json(request): Json<ValidateOkRequest>,
) -> AppResult<(), ValidateOkError> {
    let assignments = sqlx::query_as_unchecked!(
        Assignment,
        r#"
            SELECT
                *
            FROM
                assignments
            WHERE
                id = ANY($1)
        "#,
        request.assignment_ids
    )
    .fetch_all(&state.pool)
    .await?;
    println!(
        "Request: {}\t db: {}",
        request.assignment_ids.len(),
        assignments.len()
    );
    if assignments.len() != request.assignment_ids.len() {
        Err(ValidateOkError::InvalidAssignment)?;
    }

    let mut task_ids = Vec::from_iter(assignments.into_iter().map(|assignment| assignment.task_id));
    task_ids.sort();
    task_ids.dedup();

    if task_ids.len() > 1 || task_ids.is_empty() {
        Err(ValidateOkError::AssignmentTaskRelationshipError)?;
    }

    let task = Task::select_one(task_ids[0]).fetch_one(&state.pool).await?;

    let project = Project::select_one(task.project_id)
        .fetch_one(&state.pool)
        .await?;

    if task.canonical_result_id.is_some() {
        Err(ValidateOkError::CanonicalResultExists)?
    }
    if request.assignment_ids.len() != project.quorum as usize {
        Err(ValidateOkError::ResultCountQuorumNotEqual)?
    }

    sqlx::query_unchecked!(
        r#"
        UPDATE 
            tasks
        SET 
            canonical_result_id =  
            (SELECT 
                r.id 
             FROM 
                results r
             JOIN assignments a
                ON a.id = r.assignment_id
             WHERE a.id = ANY($2)
             ORDER BY 
                r.created_at DESC 
             LIMIT 1
            )
        WHERE
            id = $1
        "#,
        task.id,
        request.assignment_ids
    )
    .execute(&state.pool)
    .await?;

    set_assignment_state::set_assignment_state(
        &state,
        AssignmentState::Valid,
        &request.assignment_ids,
    )
    .await?;

    //Set assignments with results other than the ones we just set to be valid to be invalid
    let assignments_other = sqlx::query_as_unchecked!(
        Assignment,
        r#"
            SELECT
                a.*
            FROM
                assignments a
            LEFT JOIN results r
                ON r.assignment_id = a.id
            WHERE
                a.state != $1
                AND task_id = $2
        "#,
        AssignmentState::Valid,
        task.id
    )
    .fetch_all(&state.pool)
    .await?;

    set_assignment_state::set_assignment_state(
        &state,
        AssignmentState::Invalid,
        &Vec::from_iter(assignments_other.iter().map(|assignment| assignment.id)),
    )
    .await?;

    Ok(())
}
