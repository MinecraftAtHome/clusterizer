use axum::{Json, extract::State};
use clusterizer_common::{
    errors::ValidateSubmitError,
    records::{Result, Select, Task, result::UpdateResult, task::UpdateTask},
    requests::ValidateSubmitRequest,
    types::{Id, ResultState},
};

use std::collections::HashMap;

use crate::{
    auth::Auth,
    result::{AppError, AppResult},
    state::AppState,
};

pub async fn validate_submit(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(request): Json<ValidateSubmitRequest>,
) -> AppResult<(), ValidateSubmitError> {
    // Fetch results from the request.
    let result_ids: Vec<_> = request.results.keys().collect();

    let mut task_ids = sqlx::query_scalar_unchecked!(
        r#"
        SELECT
            a.task_id "task_id: Id<Task>"
        FROM
            results r,
            assignments a
        WHERE
            r.id = ANY($1)
            AND a.id = r.assignment_id
        "#,
        result_ids,
    )
    .fetch_all(&state.pool)
    .await?;

    // Check all result ids were valid.
    if task_ids.len() != request.results.len() {
        Err(AppError::Specific(ValidateSubmitError::InvalidResult))?;
    }

    // Deduplicate task ids.
    task_ids.sort();
    task_ids.dedup();

    // Can only validate one task at a time, for now.
    if task_ids.len() != 1 {
        Err(AppError::Specific(ValidateSubmitError::InvalidTaskCount))?;
    }

    let mut tx = state.pool.begin().await?;

    // Fetch tasks for the results we are going to validate.
    let task = sqlx::query_as_unchecked!(
        Task,
        r#"
        SELECT
            *
        FROM
            tasks
        WHERE
            id = $1
        FOR UPDATE
        "#,
        task_ids[0],
    )
    .fetch_one(&mut *tx)
    .await?;

    // Check project permissions.
    let project = task.project_id.select().fetch_one(&mut *tx).await?;

    if project.created_by_user_id != user_id {
        Err(AppError::Specific(ValidateSubmitError::Forbidden))?;
    }

    // Fetch the results for this task. This ignores results whose id exceeds the last id from the
    // validation request, because the validator program also did not consider them.
    let last_result_id = request
        .results
        .keys()
        .max()
        .expect("results cannot be empty");

    let results = sqlx::query_as_unchecked!(
        Result,
        r#"
        SELECT
            r.*
        FROM
            results r,
            assignments a
        WHERE
            a.task_id = $1
            AND a.id = r.assignment_id
            AND r.id <= $2
        "#,
        task.id,
        last_result_id,
    )
    .fetch_all(&mut *tx)
    .await?;

    // Build groups and errored results.
    let mut groups: HashMap<_, Vec<_>> = HashMap::new();
    let mut error_result_ids = Vec::new();

    for result in &results {
        if let Some(&group_id) = request.results.get(&result.id) {
            if result.state != ResultState::Init {
                // Error if the result was already validated.
                Err(AppError::Specific(
                    ValidateSubmitError::ForbiddenStateTransition,
                ))?;
            } else if let Some(group_id) = group_id {
                groups.entry(group_id).or_default().push(result.id);
            } else {
                error_result_ids.push(result.id);
            }
        } else if result.state == ResultState::Init {
            // The validator missed the task. This must be an error for deterministic validation.
            Err(AppError::Specific(ValidateSubmitError::MissingResults))?;
        } else if let Some(group_id) = result.group_result_id {
            groups.entry(group_id).or_default().push(result.id);
        }
    }

    // Check that each group id is the lowest of any result ids in the group.
    for (group_id, result_ids) in &groups {
        if group_id != result_ids.iter().min().expect("group cannot be empty") {
            Err(AppError::Specific(ValidateSubmitError::InconsistentGroup))?;
        }
    }

    // Update state of error results.
    error_result_ids
        .update_state(ResultState::Error)
        .execute(&mut *tx)
        .await?;

    // Update group ids.
    for (&result_id, &group_id) in &request.results {
        if let Some(group_id) = group_id {
            result_id
                .update_group_result_id(Some(group_id))
                .execute(&mut *tx)
                .await?;
        }
    }

    // Find the id of a group that meets quorum, if any. When multiple groups meet quorum, we
    // select the one with the lowest id instead of the largest group. This is needed for
    // deterministic validation.
    let valid_group_id = groups
        .iter()
        .filter(|(_, results)| results.len() as i32 >= task.quorum)
        .map(|(&group_id, _)| group_id)
        .min();

    if let Some(valid_group_id) = valid_group_id {
        // If there was a valid group, update the state of all results.
        let group_result_ids: Vec<_> = groups.values().flatten().collect();

        sqlx::query_unchecked!(
            r#"
            UPDATE
                results
            SET
                state = CASE
                    WHEN group_result_id = $1 THEN 'valid'::result_state
                    ELSE 'invalid'::result_state
                END
            WHERE
                id = ANY($2)
            "#,
            valid_group_id,
            group_result_ids,
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // Otherwise, update the state of the new results to 'inconclusive'.
        let inconclusive_result_ids: Vec<_> = request
            .results
            .iter()
            .filter(|(_, group_id)| group_id.is_some())
            .map(|(&result_id, _)| result_id)
            .collect();

        inconclusive_result_ids
            .update_state(ResultState::Inconclusive)
            .execute(&mut *tx)
            .await?;

        // Finally, update the number of assignments needed.
        let largest_inconclusive_group = groups
            .values()
            .max_by_key(|results| results.len())
            .expect("there is at least one group");

        let assignments_needed =
            (results.len() - largest_inconclusive_group.len()) as i32 + task.quorum;

        task.id
            .update_assignments_needed(assignments_needed)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(())
}
