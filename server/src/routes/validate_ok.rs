use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    errors::ValidateOkError,
    id::Id,
    requests::ValidateOkRequest,
    types::{AssignmentState, Project, Result, Task},
};

use crate::{query::SelectOne, result::AppResult, state::AppState, util::set_assignment_state};

pub async fn validate_ok(
    State(state): State<AppState>,
    Path(task_id): Path<Id<Task>>,
    Json(request): Json<ValidateOkRequest>,
) -> AppResult<(), ValidateOkError> {
    let task = Task::select_one(task_id).fetch_one(&state.pool).await?;

    let project = Project::select_one(task.project_id)
        .fetch_one(&state.pool)
        .await?;

    let result_count = sqlx::query_scalar_unchecked!(
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
        task_id
    )
    .fetch_one(&state.pool)
    .await? as usize;
    let result_task_count = sqlx::query_scalar_unchecked!(
        r#"
                SELECT
                    count(1) as "count!"
                FROM
                    results r
                WHERE
                    r.id = ANY($1)
            "#,
        request.result_ids
    )
    .fetch_one(&state.pool)
    .await? as usize;

    if task.canonical_result_id.is_some() {
        Err(ValidateOkError::CanonicalResultExists)?
    }
    if request.result_ids.len() != project.quorum as usize {
        Err(ValidateOkError::ResultCountQuorumNotEqual)?
    }
    if request.result_ids.len() != result_task_count {
        Err(ValidateOkError::ResultTaskRelationshipInconsistent)?
    }
    if request.result_ids.len() != result_count {
        Err(ValidateOkError::InvalidResult)?
    }

    sqlx::query_unchecked!(
        r#"
        UPDATE 
            tasks
        SET 
            canonical_result_id =  
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
        task_id,
        request.result_ids
    )
    .execute(&state.pool)
    .await?;

    let results = sqlx::query_as_unchecked!(
        Result,
        r#"
        SELECT 
            r.*
        FROM 
            results r
        WHERE
            r.id = ANY($1)
        "#,
        request.result_ids
    )
    .fetch_all(&state.pool)
    .await?;

    set_assignment_state::set_assignment_state(
        &state,
        AssignmentState::Validated,
        &Vec::from_iter(results.iter().map(|result| result.assignment_id)),
    )
    .await?;
    Ok(())
}
