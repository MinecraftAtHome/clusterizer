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

    let task_assignment_map = assignments.iter().fold(
        HashMap::new(),
        |mut map: HashMap<Id<Task>, Vec<Id<Assignment>>>, assignment| {
            map.entry(assignment.task_id)
                .or_default()
                .push(assignment.id);
            map
        },
    );

    for task_id in task_assignment_map.keys() {
        let task = Select::select_one(*task_id).fetch_one(&state.pool).await?;

        let task_db_assignments = task_assignment_map[task_id].clone();

        let mut group_map: HashMap<i32, i32> = HashMap::new();
        let mut errored_assignments: Vec<Id<Assignment>> = Vec::new();
        for assignment in task_db_assignments.iter() {
            let group_num = request.assignments[assignment];
            if group_num.is_some() {
                *group_map.entry(group_num.unwrap()).or_insert(0) += 1;
            } else {
                // Task errored, so add one to assignments needed
                errored_assignments.push(*assignment);
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
        }
        if valid_groups.len() == 1 || task.canonical_result_id.is_some() {
            // Valid

            let valid_assignments: Vec<Id<Assignment>> = task_db_assignments
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

            let invalid_assignments: Vec<Id<Assignment>> = task_db_assignments
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
                    ValidateSubmitError::InvalidAssignmentState,
                ))?
            }

            let inconclusive_assignments: Vec<Id<Assignment>> = task_db_assignments
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
