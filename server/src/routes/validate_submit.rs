use axum::{Json, extract::State};
use clusterizer_common::{
    errors::ValidateSubmitError,
    records::{Assignment, Result, Task},
    requests::ValidateSubmitRequest,
    types::{AssignmentState, Id},
};

use std::collections::{HashMap, HashSet};

use crate::{
    result::{AppError, AppResult},
    state::AppState,
    util::{Select, set_assignment_state},
};

pub async fn validate_submit(
    State(state): State<AppState>,
    Json(request): Json<ValidateSubmitRequest>,
) -> AppResult<(), ValidateSubmitError> {
    /*
        check that there exists no results in the db submitted before the latest given assignment id, this is a subtle issue i just thought of that i think we have not discussed before.
        find the valid group, if there is one, the valid group is the group that meets quorum with the earliest submitted result id
        if there is no valid group:
        8.1. set all assignments that are in any group to 'inconclusive'
        8.2. set assignments_needed to the number of 'inconclusive' and 'error' results plus quorum minus the size of the largest group (i think this formula is correct but unsure, we discussed it before as well but don't wanna search for the message rn)
        if there is a valid group:
        9.1. set all assignments in that group to 'valid' and all assignments in other groups to 'invalid'
        9.2. set the canonical result id to the earliest result in the group
        import to note that when talking about groups, we should always also consider groups that were already in the db
        not just the groups that the validator just submitted



    */
    let mut group_ids: HashSet<Id<Result>> = HashSet::new();
    let mut assignment_ids: Vec<Id<Assignment>> = request.assignments.keys().cloned().collect();
    let mut group_id_by_assignment: HashMap<Id<Assignment>, Id<Result>> = HashMap::new();
    let mut assignments_by_group_id: HashMap<Id<Result>, Vec<Id<Assignment>>> = HashMap::new();

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
        &assignment_ids
    )
    .fetch_all(&state.pool)
    .await?;
    // Ensure all assignments are real
    if assignment_ids.len() != assignments.len() {
        Err(AppError::Specific(ValidateSubmitError::InvalidAssignment))?
    }

    // Ensure all assignments are for the same task
    let task_id = assignments[0].task_id;
    if assignments.iter().any(|ass| ass.task_id != task_id) {
        Err(AppError::Specific(
            ValidateSubmitError::TooManyTasksValidationError,
        ))?
    }
    // Disallow state transitions via validation unless the assignment is in the Submitted state
    if assignments
        .iter()
        .any(|assignment| assignment.state != AssignmentState::Submitted)
    {
        Err(AppError::Specific(
            ValidateSubmitError::StateTransitionForbidden,
        ))?
    }

    // Set assignments to Error if they do not have a group_id (aka result_id)
    for (ass, group_id) in request.assignments {
        match group_id {
            Some(g) => {
                // Add group id for that assignment to group_ids
                // Add assignment id to assignment_ids
                // Add assignment id and group id to new HashMap which filters out errored results
                assignments_by_group_id
                    .entry(g)
                    .or_insert_with(Vec::new)
                    .push(ass);
                group_ids.insert(g);
                group_id_by_assignment.insert(ass, g);
            }
            None => {
                set_assignment_state(&[ass], AssignmentState::Error)
                    .execute(&state.pool)
                    .await?;
            }
        }
    }
    let assignment_by_id: HashMap<Id<Assignment>, &Assignment> =
        assignments.iter().map(|ass| (ass.id, ass)).collect();

    let task = Task::select_one(task_id).fetch_one(&state.pool).await?;

    for (group_id, group_assignment_ids) in assignments_by_group_id {
        let group_db_results: Vec<Result> = sqlx::query_as_unchecked!(
            Result,
            r#"
            SELECT
                *
            FROM
                results
            WHERE
                group_result_id = $1
            "#,
            &group_id
        )
        .fetch_all(&state.pool)
        .await?;

        let mut group_results = sqlx::query_as_unchecked!(
            Result,
            r#"
            SELECT
                *
            FROM
                results
            WHERE
                assignment_id = ANY($1)
            "#,
            &group_assignment_ids
        )
        .fetch_all(&state.pool)
        .await?;
        // Get result ids before extending with existing db values so we aren't setting rows that don't need to be set
        let result_ids: Vec<Id<Result>> = group_results.iter().map(|result| result.id).collect();
        group_results.extend(group_db_results);
        // Earliest submitted result within the group, db or fresh validator data
        let group_canonical_result = group_results
            .iter()
            .min_by_key(|result| result.created_at)
            .expect("These are all known to be real already");
        // Set validator-provided results to the same group id
        sqlx::query_unchecked!(
            r#"
            UPDATE 
                results
            SET 
                group_result_id = $1
            WHERE 
                id = ANY($2)
            "#,
            group_canonical_result.id,
            &result_ids
        )
        .execute(&state.pool)
        .await?;

        // Check if we have quorum
        if (group_results.len() as i32) >= task.quorum {
            // Met quorum
            // This should also catch the case that the db results + new results = quorum or higher since we combine them in an earlier step
            match task.canonical_result_id {
                Some(_) => {}
                None => {
                    // Set canonical result
                    sqlx::query_unchecked!(
                        r#"
                        UPDATE 
                            tasks
                        SET 
                            canonical_result_id = $1
                        WHERE 
                            id = $2
                        "#,
                        group_canonical_result.id,
                        task.id
                    )
                    .execute(&state.pool)
                    .await?;
                }
            }
            // Set to valid
            set_assignment_state(&group_assignment_ids, AssignmentState::Valid)
                .execute(&state.pool)
                .await?;
            // Invalidate other groups for this task
        } else if let Some(canonical_result_id) = task.canonical_result_id
            && group_id != canonical_result_id
        {
            // Invalid
            set_assignment_state(&group_assignment_ids, AssignmentState::Invalid)
                .execute(&state.pool)
                .await?;
        } else {
            // Inconclusive
            set_assignment_state(&group_assignment_ids, AssignmentState::Inconclusive)
                .execute(&state.pool)
                .await?;
            // Find largest group for task
            let mut group_id_count: HashMap<Id<Result>, i32> = HashMap::new();
            for gr in group_results {
                match gr.group_result_id {
                    Some(gr_id) => *group_id_count.entry(gr_id).or_insert(0) += 1,
                    None => {}
                }
            }
            let largest_group_size: i32 = group_id_count
                .iter()
                .filter_map(|g| group_id_count.get(g).copied())
                .max()
                .unwrap_or(0);
            // Set assignments_needed
            sqlx::query_unchecked!(
                r#"
                UPDATE 
                    tasks
                SET 
                    assignments_needed = $1
                WHERE 
                    id = $2
                "#,
                task.quorum - largest_group_size + task.id
            )
            .execute(&state.pool)
            .await?;
        }
    }
    Ok(())
}
