use axum::{Json, extract::State};
use clusterizer_common::{
    errors::ValidateSubmitError,
    records::{Result, Task},
    requests::ValidateSubmitRequest,
    types::ResultState,
};

use std::collections::HashMap;

use crate::{
    result::{AppError, AppResult},
    state::AppState,
    util::set_result_state,
};

pub async fn validate_submit(
    State(state): State<AppState>,
    Json(request): Json<ValidateSubmitRequest>,
) -> AppResult<(), ValidateSubmitError> {
    // Fetch results from the request.
    let result_ids: Vec<_> = request.results.keys().collect();

    let results = sqlx::query_as_unchecked!(
        Result,
        r#"
        SELECT
            *
        FROM
            results
        WHERE
            id = ANY($1)
        "#,
        result_ids
    )
    .fetch_all(&state.pool)
    .await?;

    // Check all result ids were valid.
    if results.len() != request.results.len() {
        Err(AppError::Specific(ValidateSubmitError::InvalidResult))?
    }

    // Check all results have the 'init' state.
    if results
        .iter()
        .any(|result| result.state != ResultState::Init)
    {
        Err(AppError::Specific(
            ValidateSubmitError::ForbiddenStateTransition,
        ))?
    }

    // Fetch tasks for the results we are going to validate.
    let assignment_ids: Vec<_> = results.iter().map(|result| result.assignment_id).collect();

    let tasks = sqlx::query_as_unchecked!(
        Task,
        r#"
        SELECT
            t.*
        FROM
            tasks t
            JOIN assignments a ON
                a.task_id = t.id
        WHERE
            a.id = ANY($1)
        "#,
        assignment_ids,
    )
    .fetch_all(&state.pool)
    .await?;

    // Can only validate one task at a time, for now.
    if tasks.len() != 1 {
        Err(AppError::Specific(ValidateSubmitError::InvalidTaskCount))?
    }

    let task = &tasks[0];

    // Fetch the remaining results for this task. This ignores results whose id exceeds the largest
    // id from the validation request, because the validator program also did not consider them.
    let last_result_id = request
        .results
        .keys()
        .max()
        .expect("results cannot be empty");

    let previous_results = sqlx::query_as_unchecked!(
        Result,
        r#"
        SELECT
            r.*
        FROM
            results r
            JOIN assignments a ON
                a.id = r.assignment_id
        WHERE
            a.task_id = $1
            AND r.id < $2
            AND r.id != ALL($3)
        "#,
        task.id,
        last_result_id,
        result_ids,
    )
    .fetch_all(&state.pool)
    .await?;

    // Check the validator didn't miss any tasks. This is needed for deterministic validation.
    if previous_results
        .iter()
        .any(|result| result.state == ResultState::Init)
    {
        Err(AppError::Specific(ValidateSubmitError::MissingResults))?
    }

    // Build groups and errored results.
    let mut groups: HashMap<_, Vec<_>> = HashMap::new();
    let mut error_result_ids = Vec::new();

    for result in &previous_results {
        if let Some(group_id) = result.group_result_id {
            groups.entry(group_id).or_default().push(result.id);
        }
    }

    for (&result_id, &group_id) in &request.results {
        if let Some(group_id) = group_id {
            groups.entry(group_id).or_default().push(result_id);
        } else {
            error_result_ids.push(result_id);
        }
    }

    // Check that each group id is the lowest of any result ids in the group.
    for (group_id, result_ids) in &groups {
        if group_id != result_ids.iter().min().expect("group cannot be empty") {
            Err(AppError::Specific(ValidateSubmitError::InconsistentGroup))?
        }
    }

    // Update state of error results.
    set_result_state(&error_result_ids, ResultState::Error)
        .execute(&state.pool)
        .await?;

    // Update group ids.
    for (&result_id, &group_id) in &request.results {
        if let Some(group_id) = group_id {
            sqlx::query_unchecked!(
                r#"
                UPDATE
                    results
                SET
                    group_result_id = $1
                WHERE
                    id = $2
                "#,
                group_id,
                result_id,
            )
            .execute(&state.pool)
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
        .execute(&state.pool)
        .await?;
    } else {
        // Otherwise, update the state of the new results to 'inconclusive'.
        let inconclusive_result_ids: Vec<_> = request
            .results
            .iter()
            .filter(|(_, group_id)| group_id.is_some())
            .map(|(result_id, _)| result_id)
            .collect();

        sqlx::query_unchecked!(
            r#"
            UPDATE
                results
            SET
                state = 'inconclusive'
            WHERE
                id = ANY($1)
            "#,
            inconclusive_result_ids,
        )
        .execute(&state.pool)
        .await?;

        // Finally, update the number of assignments needed.
        let largest_inconclusive_group = groups
            .values()
            .max_by_key(|results| results.len())
            .expect("there is at least one group");

        let assignments_needed = (results.len() + previous_results.len()
            - largest_inconclusive_group.len()) as i32
            + task.quorum;

        sqlx::query_unchecked!(
            r#"
            UPDATE
                tasks
            SET
                assignments_needed = $1
            WHERE
                id = $2
            "#,
            assignments_needed,
            task.id,
        )
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}
