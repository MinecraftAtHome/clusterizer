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

    let assignment_by_id: HashMap<Id<Assignment>, &Assignment> =
        assignments.iter().map(|ass| (ass.id, ass)).collect();

    // Disallow state transitions via validation unless the assignment is one of these states
    if assignments.iter().any(|assignment| {
        assignment.state != AssignmentState::Submitted
            && assignment.state != AssignmentState::Inconclusive
    }) {
        Err(AppError::Specific(
            ValidateSubmitError::StateTransitionForbidden,
        ))?
    }
    /*
        1. Obtain unique group number from group number, split request into assignment_ids split by their group number
        2. Error if assignments for different tasks were given the same group number
        3. Error if assignments for the same task were given different group ids but those group ids are the other assignment
            { "1": 2, "2": 1 }
            Without changing anything it should just think quorum hasn't been met.
            The special case is that it's a circular reference.
            Check that
        Per group processing:
            1. Determine which assignment within a group submitted a result first, that's the potential canonical result
            2. Determine if enough results are within that group to meet quorum
                a. if there are
                    1. Mark the assignments as valid.
                    2. Set task.canonical_result = the result determined in #1
                    3. Set all other assignments for that task that are in different groups to invalid
                b. if there are not
                    1. Set all assignments to inconclusive
                    2. Increment assignments_needed by 1



    */

    // 1. Start with assignment_id to group_number hashmap
    // Need to create a vec of group_number and dedup it
    let mut group_ids: Vec<Id<Assignment>> = request.assignments.values().copied().collect();
    group_ids.sort();
    group_ids.dedup();
    //Create inverse of request - group_id first, then vec of assignment_ids
    let mut group_assignment_map: HashMap<Id<Assignment>, Vec<Id<Assignment>>> = HashMap::new();
    for group_id in group_ids {
        group_assignment_map.entry(group_id).or_default().extend(
            request
                .assignments
                .iter()
                .filter(|x| *x.1 == group_id)
                .map(|x| *x.1),
        );
        //Error checking
        let mut task_unique: HashSet<Id<Task>> = HashSet::new();
        for assignment_id in &group_assignment_map[&group_id] {
            if let Some(a) = assignments.iter().find(|a| &a.id == assignment_id) {
                task_unique.insert(a.task_id);
            }
            if request.assignments[&request.assignments[assignment_id]]
                != request.assignments[assignment_id]
            {
                Err(AppError::Specific(
                    ValidateSubmitError::ValidationGroupAssociationInconsistency,
                ))?
            }
        }
        if task_unique.len() > 1 {
            Err(AppError::Specific(
                ValidateSubmitError::ValidationGroupTaskInconsistency,
            ))?
        }
        let task: Task = Task::select_one(
            task_unique
                .iter()
                .next()
                .copied() // or `.cloned()` if Id<Task> isn't Copy
                .expect("task_unique must contain exactly one task_id"),
        )
        .fetch_one(&state.pool)
        .await?;
        //Are there enough for quorum
        if (group_assignment_map[&group_id].len() as i32) < task.quorum {
            // Inconclusive
            let ids = &group_assignment_map[&group_id];
            let by_id: HashMap<Id<Assignment>, AssignmentState> =
                assignments.iter().map(|a| (a.id, a.state)).collect();
            if ids.iter().all(|&aid| {
                matches!(
                    by_id.get(&aid).copied().expect("assignment id must exist"),
                    AssignmentState::Inconclusive
                )
            }) {
                // Cannot run the inconclusive part if we've already submitted a new one.
                Err(AppError::Specific(
                    ValidateSubmitError::ValidationImpossibleError,
                ))?
            }
            set_assignment_state(
                &group_assignment_map[&group_id],
                AssignmentState::Inconclusive,
            )
            .execute(&state.pool)
            .await?;
            sqlx::query_unchecked!(
                r#"
            UPDATE
                tasks
            SET
                assignments_needed = assignments_needed + 1
            WHERE
                id = $1
            
            "#,
                task.id
            )
            .execute(&state.pool)
            .await?;

            break;
        }
        // There are enough for quorum
        // Get assignments for our task_id, regardless of group
        let task_assignments: Vec<Assignment> = assignments
            .iter()
            .filter(|ass| ass.task_id == task.id)
            .cloned()
            .collect();
        // Get their Ids
        let task_assignment_ids: Vec<Id<Assignment>> =
            task_assignments.iter().map(|ass| ass.id).collect();
        // Use their ids to get the results for each
        let task_results: Vec<Result> = sqlx::query_as_unchecked!(
            Result,
            r#"
            SELECT
                *
            FROM
                results
            WHERE
                assignment_id = ANY($1)
            "#,
            &task_assignment_ids
        )
        .fetch_all(&state.pool)
        .await?;
        // Get results only for this group
        let group_results: Vec<Result> = task_results
            .iter()
            .filter(|res| group_assignment_map[&group_id].contains(&res.assignment_id))
            .cloned()
            .collect();
        // Get the earliest submitted result within that group
        let earliest_group_result = group_results
            .iter()
            .min_by_key(|r| r.created_at)
            .cloned()
            .expect("this should not be hit since we're only grabbing the min");
        let mut relevant_task_results: Vec<&Result> = Vec::new();

        // iterate over task results to find other groups with the same task
        for res in task_results.iter() {
            if let Some(ass) = assignment_by_id.get(&res.assignment_id) {
                // 1. Its actual group_id is not the current_group_id
                // 2. It's associated with the target task.id
                if ass.id != group_id && ass.task_id == task.id {
                    relevant_task_results.push(res);
                }
            }
        }

        let earliest_relevant_result: Option<&Result> = relevant_task_results
            .iter()
            .min_by_key(|r| r.created_at)
            .copied();
        if let Some(earliest_result) = earliest_relevant_result {
            // There is another result for another group for this task
            if earliest_group_result.created_at <= earliest_result.created_at {
                // This can be canonical
                // Set group to valid
                set_assignment_state(&group_assignment_map[&group_id], AssignmentState::Valid)
                    .execute(&state.pool)
                    .await?;
                // Set task's canonical result id
                sqlx::query_unchecked!(
                    r#"
                    UPDATE
                        tasks
                    SET
                        canonical_result_id = $1
                    WHERE
                        id = $2
                    "#,
                    earliest_group_result.id,
                    task.id
                )
                .execute(&state.pool)
                .await?;
            }
        } else {
            // This is the only relevant group, set to valid
            set_assignment_state(&group_assignment_map[&group_id], AssignmentState::Valid)
                .execute(&state.pool)
                .await?;
            // Set task's canonical result id
            sqlx::query_unchecked!(
                r#"
                UPDATE
                    tasks
                SET
                    canonical_result_id = $1
                WHERE
                    id = $2
                "#,
                earliest_group_result.id,
                task.id
            )
            .execute(&state.pool)
            .await?;
        }
    }
    Ok(())
}
