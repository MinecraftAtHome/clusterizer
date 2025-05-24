use axum::{Json, extract::State};
use clusterizer_common::{
    errors::ValidateSubmitError,
    records::{Assignment, Task},
    requests::ValidateSubmitRequest,
    types::{AssignmentState, Id},
};

use std::collections::HashMap;

use crate::{
    result::{AppError, AppResult},
    state::AppState,
    util::{Select, set_assignment_state},
};

pub async fn validate_submit(
    State(state): State<AppState>,
    Json(request): Json<ValidateSubmitRequest>,
) -> AppResult<(), ValidateSubmitError> {
    let assignment_ids: Vec<_> = request.assignments.keys().copied().collect();

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
        assignment_ids
    )
    .fetch_all(&state.pool)
    .await?;

    if assignment_ids.len() != assignments.len() {
        Err(AppError::Specific(ValidateSubmitError::InvalidAssignment))?
    }

    // Disallow state transitions via validation unless the assignment is one of these states
    if assignments.iter().any(|assignment| {
        assignment.state != AssignmentState::Submitted
            && assignment.state != AssignmentState::Inconclusive
    }) {
        Err(AppError::Specific(
            ValidateSubmitError::StateTransitionForbidden,
        ))?
    }

    // Generate a map of tasks to assignments with that task_id for use in the next loop
    let mut task_assignment_map: HashMap<Id<Task>, Vec<Id<Assignment>>> = HashMap::new();

    for assignment in &assignments {
        task_assignment_map
            .entry(assignment.task_id)
            .or_default()
            .push(assignment.id);
    }
    if task_assignment_map.keys().len() > 1 {
        Err(AppError::Specific(
            ValidateSubmitError::MultipleTasksDisallowed,
        ))?
    }
    // Loop through all tasks that have been submitted for validation via their assignments
    // This avoids a MixedTasks error as we simply error with "too few results" instead for that particular task
    for (task_id, task_assignments) in &task_assignment_map {
        let task = Task::select_one(*task_id).fetch_one(&state.pool).await?;

        let mut group_map: HashMap<i32, Vec<Id<Assignment>>> = HashMap::new();
        let mut errored_assignments: Vec<Id<Assignment>> = Vec::new();

        // Get group number and count of assignments in each group
        for assignment in task_assignments {
            let group_num_option = request.assignments[assignment];

            match group_num_option {
                // Task has a group number, so use it.
                Some(group_num) => group_map.entry(group_num).or_default().push(*assignment),
                // Task errored, so add one to assignments needed
                None => errored_assignments.push(*assignment),
            }
        }

        let valid_groups: HashMap<i32, Vec<Id<Assignment>>> = group_map
            .iter()
            .filter(|(_, assignments)| assignments.len() as i32 >= task.quorum)
            .map(|(group_num, assignments)| (*group_num, assignments.clone()))
            .collect();

        let invalid_groups: HashMap<i32, Vec<Id<Assignment>>> = group_map
            .iter()
            .filter(|(_, assignments)| (assignments.len() as i32) < task.quorum)
            .map(|(group_num, assignments)| (*group_num, assignments.clone()))
            .collect();

        if valid_groups.len() == 1 || task.canonical_result_id.is_some() {
            // Valid
            let valid_assignments = &valid_groups[&0];
            let mut invalid_assignments = Vec::new();
            for invalid_assignment_ids in invalid_groups.values() {
                for invalid_assignment_id in invalid_assignment_ids {
                    invalid_assignments.push(*invalid_assignment_id);
                }
            }

            if task.canonical_result_id.is_none() {
                // If we don't have a valid result yet

                let canonical_result = sqlx::query_as_unchecked!(
                    clusterizer_common::records::Result,
                    r#"
                    SELECT 
                        *
                    FROM 
                        results r
                    WHERE 
                        r.assignment_id = ANY($1)
                    ORDER BY 
                        r.created_at ASC
                    LIMIT 1
                    "#,
                    valid_assignments
                )
                .fetch_one(&state.pool)
                .await?;

                sqlx::query_unchecked!(
                    r#"
                    UPDATE 
                        tasks
                    SET 
                        canonical_result_id = $2
                    WHERE
                        id = $1
                    "#,
                    task.id,
                    canonical_result.id
                )
                .execute(&state.pool)
                .await?;
            }
            set_assignment_state::set_assignment_state(valid_assignments, AssignmentState::Valid)
                .execute(&state.pool)
                .await?;

            // Mark invalid
            set_assignment_state::set_assignment_state(
                &invalid_assignments,
                AssignmentState::Invalid,
            )
            .execute(&state.pool)
            .await?;
        }
        if valid_groups.len() > 1 || !invalid_groups.is_empty() {
            // Either no groups met quorum, or more than one did and we need to break the tie.
            // Inconclusive

            // Cannot be inconclusive if a result is canonical already. Either it's valid or it's not.
            if task.canonical_result_id.is_some() {
                Err(AppError::Specific(
                    ValidateSubmitError::InconsistentValidationState,
                ))?
            }
            // If there are two or more valid groups, submit another assignment and hope that breaks the tie.
            if valid_groups.len() > 1 {
                sqlx::query_unchecked!(
                    r#"
                    UPDATE 
                        tasks
                    SET 
                        assignments_needed = assignments_needed + 1
                    WHERE
                        id = $1
                    "#,
                    task.id,
                )
                .execute(&state.pool)
                .await?;
                for (_, inconclusive_assignments) in valid_groups {
                    // Mark assignments as inconclusive
                    set_assignment_state(&inconclusive_assignments, AssignmentState::Inconclusive)
                        .execute(&state.pool)
                        .await?;
                }
            } else {
                let mut invalid_assignments = Vec::new();
                for (_, invalid_assignment_ids) in invalid_groups {
                    for invalid_assignment_id in invalid_assignment_ids {
                        invalid_assignments.push(invalid_assignment_id);
                    }
                }

                let mut max_count = 0;
                for (_, assignments) in group_map {
                    if assignments.len() as i32 > max_count {
                        max_count = assignments.len() as i32;
                    }
                }
                sqlx::query_unchecked!(
                    r#"
                    UPDATE 
                        tasks
                    SET 
                        assignments_needed = assignments_needed + $2
                    WHERE
                        id = $1
                    "#,
                    task.id,
                    task.quorum - max_count
                )
                .execute(&state.pool)
                .await?;

                // Mark assignments as inconclusive
                set_assignment_state(&invalid_assignments, AssignmentState::Inconclusive)
                    .execute(&state.pool)
                    .await?;

                // Mark assignments as errored
                set_assignment_state(&errored_assignments, AssignmentState::Error)
                    .execute(&state.pool)
                    .await?;
            }
        }
    }

    Ok(())
}
