use axum::{Json, extract::State};
use clusterizer_common::{
    errors::ValidateSubmitError,
    records::{Result, Task},
    requests::ValidateSubmitRequest,
    types::{Id, ResultState},
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
    /*
        given_results = fetch all results of given result_ids

        if given_results.len() != result_ids.len():
            error

        if the state of any of given_results is not 'init':
            error

        tasks = fetch all tasks of given_results

        if tasks.len() != 1:
            error

        last_date = max(given_results submitted date)

        previously_given_results = fetch all results submitted before last_date and not in given_results

        if the state of any of previously_given_results is 'init':
            error

        if the group id of the result with the same id as any given group id is not equal to that same group id:
            error

        error_results = []

        for result, group_id in given_results:
            if group_id is not None:
                update group id of result
                result.group_id = group_id
            else:
                error_results.append(result)
                result.state = 'error'

        update state of error_results

        all_results = given_results + previously_given_results

        results_by_group_id = you know how to make this

        valid_group_id = group_id of the group with at least quorum reuslts that has the earliest submitted result

        if valid_group_id is None:
            set result state of all results in results_by_group_id to 'inconslusive'
            update assignments_needed
        else:
            UPDATE
                results
            SET
                state = (group_id = $1 ? 'valid' : 'invalid')
            WHERE
                id = ANY($2)

            # for group_id, group_result_ids in results_by_group_id:
            #     if group_id == valid_group_id:
            #         set result state to 'valid'
            #     else:
            #         set result state to 'invalid'

    */
    let result_ids: Vec<_> = request.results.keys().cloned().collect();

    let given_results: Vec<clusterizer_common::records::Result> = sqlx::query_as_unchecked!(
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

    if given_results.len() != request.results.len() {
        Err(AppError::Specific(ValidateSubmitError::InvalidResult))?
    }

    if given_results
        .iter()
        .any(|result| result.state != ResultState::Init)
    {
        Err(AppError::Specific(
            ValidateSubmitError::ForbiddenStateTransition,
        ))?
    }

    let given_tasks: Vec<Task> = sqlx::query_as_unchecked!(
        Task,
        r#"
        SELECT
            t.*
        FROM
            tasks t
        JOIN
            assignments a
        ON
            a.task_id = t.id
        WHERE
            a.id = ANY($1)
        "#,
        given_results
            .iter()
            .map(|result| result.assignment_id)
            .collect::<Vec<_>>()
    )
    .fetch_all(&state.pool)
    .await?;

    if given_tasks.len() != 1 {
        Err(AppError::Specific(ValidateSubmitError::InvalidTaskCount))?
    }

    let task = &given_tasks[0];
    let last_given_date = given_results
        .clone()
        .iter()
        .map(|result| result.created_at)
        .max()
        .expect("There will be a created_at because of the db schema");

    let previously_given_results: Vec<Result> = sqlx::query_as_unchecked!(
        Result,
        r#"
        SELECT
            r.*
        FROM
            results r
        JOIN
            assignments a
        ON
            a.id = r.assignment_id
        WHERE
            a.task_id = $1
        AND
            r.created_at < $2
        AND
            r.id != ALL($3)
        "#,
        task.id,
        last_given_date,
        given_results
            .clone()
            .into_iter()
            .map(|result| result.id)
            .collect::<Vec<_>>()
    )
    .fetch_all(&state.pool)
    .await?;

    // if the state of any of previously_given_results is 'init':
    if previously_given_results
        .iter()
        .any(|result| result.state == ResultState::Init)
    {
        Err(AppError::Specific(
            ValidateSubmitError::ForbiddenStateTransition,
        ))?
    }
    let mut all_given_results = previously_given_results.clone();
    all_given_results.extend(given_results.clone());

    let mut results_by_group_id: HashMap<Id<Result>, Vec<Id<Result>>> = HashMap::new();
    // Build results_by_group_id
    for (result_id, group_id) in &request.results {
        if let Some(gid) = group_id {
            results_by_group_id
                .entry(*gid)
                .or_default()
                .push(*result_id);
        }
    }

    for (group_id, results) in &results_by_group_id {
        if group_id != results.iter().min().expect("group cannot be empty") {
            Err(AppError::Specific(
                ValidateSubmitError::NondeterministicGroup,
            ))?
        }
    }

    let mut results_by_result_id: HashMap<Id<Result>, Result> = HashMap::new();
    for result in all_given_results.clone() {
        results_by_result_id.insert(result.id, result);
    }

    for (result_id, group_id) in request.results {
        match group_id {
            Some(gid) => {
                // Validation successful, update group_result_id both locally and in db
                results_by_result_id
                    .get_mut(&result_id)
                    .unwrap()
                    .group_result_id = Some(gid);
                sqlx::query_unchecked!(
                    r#"
                    UPDATE
                        results
                    SET
                        group_result_id = $1
                    WHERE
                        id = $2
                    "#,
                    Some(gid.clone()),
                    result_id
                )
                .execute(&state.pool)
                .await?;
            }
            None => {
                // Validation unsuccessful, update state to error
                results_by_result_id.get_mut(&result_id).unwrap().state = ResultState::Error;
                set_result_state(&[result_id], ResultState::Error)
                    .execute(&state.pool)
                    .await?;
            }
        }
    }

    let valid_group_id = results_by_group_id
        .iter()
        .filter(|(_, results)| results.len() as i32 >= task.quorum)
        .map(|(&group_id, results)| {
            let earliest = results
                .iter()
                .map(|r| results_by_result_id[r].created_at)
                .min()
                .expect("groups are never empty");
            (group_id, earliest)
        })
        .min_by_key(|&(_, earliest)| earliest)
        .map(|(group_id, _)| group_id);

    if valid_group_id.is_none() {
        sqlx::query_unchecked!(
            r#"
        UPDATE
            results
        SET
            state = 'inconclusive'
        WHERE
            id = ANY($1)
        "#,
            all_given_results
                .clone()
                .into_iter()
                .map(|result| result.id)
                .collect::<Vec<_>>()
        )
        .execute(&state.pool)
        .await?;

        let mut group_counts: HashMap<Id<Result>, i32> = HashMap::new();
        for mut result in all_given_results {
            if given_results
                .clone()
                .into_iter()
                .any(|g_r| g_r.id == result.id)
            {
                result.state = ResultState::Inconclusive;
            }
            if result.state == ResultState::Inconclusive
                && let Some(group_id) = result.group_result_id
            {
                *group_counts.entry(group_id).or_insert(0) += 1;
            }
        }

        let largest_inconclusive_group_count =
            group_counts.into_values().max().expect("It will exist");
        sqlx::query_unchecked!(
            r#"
        UPDATE
            tasks
        SET
            assignments_needed = $1
        WHERE
            id = $2
        "#,
            task.quorum - largest_inconclusive_group_count,
            task.id
        )
        .execute(&state.pool)
        .await?;
    } else {
        sqlx::query_unchecked!(
            r#"
        UPDATE
            results
        SET
            state =   CASE
                            WHEN group_result_id = $1 THEN 'valid'::result_state
                            ELSE 'invalid'::result_state
                        END
        WHERE
            id = ANY($2)
        "#,
            valid_group_id,
            all_given_results
                .into_iter()
                .map(|result| result.id)
                .collect::<Vec<_>>()
        )
        .execute(&state.pool)
        .await?;
    }
    Ok(())
}
