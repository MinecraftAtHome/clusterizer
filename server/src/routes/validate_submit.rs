use axum::{Json, extract::State};
use clusterizer_common::{
    errors::ValidateSubmitError,
    records::{Assignment, Task},
    requests::ValidateSubmitRequest,
    types::{AssignmentState, Id},
};

use std::collections::HashMap;

use crate::{
    result::{AppError, AppResult, ResultExt},
    state::AppState,
    util::{Select, set_assignment_state},
};

pub async fn validate_submit(
    State(state): State<AppState>,
    Json(request): Json<ValidateSubmitRequest>,
) -> AppResult<(), ValidateSubmitError> {
    let assignment_ids: Vec<Id<Assignment>> = request.assignments.keys().cloned().collect();

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
    .await? as Vec<Assignment>;

    if assignment_ids.len() != assignments.len() {
        Err(AppError::Specific(ValidateSubmitError::InvalidAssignment))?
    }

    let mut task_ids = Vec::from_iter(assignments.iter().map(|assignment| assignment.task_id));
    task_ids.sort();
    task_ids.dedup();

    // Generate a map of tasks to assignments with that task_id for use in the next loop
    let task_assignment_map = assignments.iter().fold(
        HashMap::new(),
        |mut map: HashMap<Id<Task>, Vec<Id<Assignment>>>, assignment| {
            map.entry(assignment.task_id)
                .or_default()
                .push(assignment.id);
            map
        },
    );

    // Loop through all tasks that have been submitted for validation via their assignments
    // This avoids a MixedTasks error as we simply error with "too few results" instead for that particular task
    for (task_id, task_assignments) in task_assignment_map.iter() {
        let task = Select::select_one(*task_id).fetch_one(&state.pool).await?;

        let mut group_map: HashMap<i32, i32> = HashMap::new();
        let mut errored_assignments: Vec<Id<Assignment>> = Vec::new();

        // Get group number and count of assignments in each group
        for assignment in task_assignments.iter() {
            let group_num_option = request.assignments[assignment];

            match group_num_option {
                // Task has a group number, so use it.
                Some(group_num) => *group_map.entry(group_num).or_insert(0) += 1,
                // Task errored, so add one to assignments needed
                None => errored_assignments.push(*assignment),
            }
        }

        let valid_groups: Vec<i32> = group_map
            .iter()
            .filter(|kvp| *kvp.1 >= task.quorum)
            .map(|kvp| *kvp.0)
            .collect();

        let invalid_groups: Vec<i32> = group_map
            .iter()
            .filter(|kvp| *kvp.1 < task.quorum)
            .map(|kvp| *kvp.0)
            .collect();

        if valid_groups.len() > 1 {
            // Cannot have more than one valid group - this is inconsistent
            Err(AppError::Specific(ValidateSubmitError::ValidityAmbiguous))?
        }
        if valid_groups.len() == 1 || task.canonical_result_id.is_some() {
            // Valid

            let valid_assignments: Vec<Id<Assignment>> = task_assignments
                .iter()
                .filter(|assignment| {
                    if let Some(group_num) = request.assignments[assignment] {
                        group_num == valid_groups[0]
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            let invalid_assignments: Vec<Id<Assignment>> = task_assignments
                .iter()
                .filter(|assignment| {
                    invalid_groups
                        .iter()
                        .any(|group_num| *group_num == request.assignments[assignment].unwrap())
                })
                .copied()
                .collect();

            // If any assignments in db have states besides Submitted or Inconclusive and they have been submitted as valid, disallow it.
            if valid_assignments.iter().any(|assignment| {
                assignments.iter().any(|ass| {
                    ass.id == *assignment
                        && ass.state != AssignmentState::Submitted
                        && ass.state != AssignmentState::Inconclusive
                }) || invalid_assignments.iter().any(|assignment| {
                    assignments.iter().any(|ass| {
                        ass.id == *assignment
                            && ass.state != AssignmentState::Submitted
                            && ass.state != AssignmentState::Inconclusive
                    })
                })
            }) {
                Err(AppError::Specific(
                    ValidateSubmitError::StateTransitionForbidden,
                ))?
            }

            if task.canonical_result_id.is_none() {
                // If we don't have a valid result yet
                sqlx::query_unchecked!(
                    r#"
                    UPDATE 
                        tasks
                    SET 
                        canonical_result_id =  
                        (
                        SELECT 
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
                    valid_assignments
                )
                .execute(&state.pool)
                .await?;
            }
            set_assignment_state::set_assignment_state(
                valid_assignments.as_slice(),
                AssignmentState::Valid,
            )
            .execute(&state.pool)
            .await
            .map_not_found(ValidateSubmitError::InvalidAssignment)?;

            // Mark invalid
            set_assignment_state::set_assignment_state(
                &invalid_assignments,
                AssignmentState::Invalid,
            )
            .execute(&state.pool)
            .await?;
        }
        if valid_groups.is_empty() && !invalid_groups.is_empty() {
            // No groups had a count at least equal to quorum
            // Inconclusive

            // Cannot be inconclusive if a result is canonical already. Either it's valid or it's not.
            if task.canonical_result_id.is_some() {
                Err(AppError::Specific(
                    ValidateSubmitError::InconsistentValidationState,
                ))?
            }

            let inconclusive_assignments: Vec<Id<Assignment>> = task_assignments
                .iter()
                .filter(|assignment| {
                    invalid_groups
                        .iter()
                        .any(|group_num| *group_num == request.assignments[assignment].unwrap())
                })
                .copied()
                .collect();

            let mut max_count = 0;
            for (_, count) in group_map {
                if count > max_count {
                    max_count = count;
                }
            }
            sqlx::query_unchecked!(
                r#"
                UPDATE tasks
                SET assignments_needed = assignments_needed + $2
                WHERE
                    id = $1
                "#,
                task.id,
                task.quorum - (errored_assignments.len() as i32 + max_count)
            )
            .execute(&state.pool)
            .await?;

            // Mark assignments as inconclusive
            set_assignment_state(&inconclusive_assignments, AssignmentState::Inconclusive)
                .execute(&state.pool)
                .await?;

            //Mark assignments as errored
            set_assignment_state(&errored_assignments, AssignmentState::Error)
                .execute(&state.pool)
                .await?;
        }
    }

    Ok(())
}
