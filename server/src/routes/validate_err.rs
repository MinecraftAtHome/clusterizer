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
    Path(task_id): Path<Id<Task>>,
    Json(request): Json<ValidateErrRequest>,
) -> AppResult<(), ValidateErrError> {
    let task = Task::select_one(task_id).fetch_one(&state.pool).await?;

    if task.canonical_result_id.is_some() {
        Err(ValidateErrError::CanonicalResultExists)?
    }

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
    .await? as i32;

    if request.assignments_needed <= 0
        || request.assignments_needed + task.assignments_needed > project.quorum + result_count
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
            task_id,
            request.assignments_needed
        )
        .execute(&state.pool)
        .await?;

        //Set validate state
        set_assignment_state::set_assignment_state(
            &state,
            AssignmentState::ValidationInconclusive,
            &Vec::from_iter(
                Assignment::select_all_by(task_id)
                    .fetch_all(&state.pool)
                    .await?
                    .iter()
                    .map(|assignment| assignment.id),
            ),
        )
        .await?;
    }

    Ok(())
}
