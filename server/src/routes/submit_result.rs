use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    errors::SubmitResultError,
    records::{Assignment, Task},
    requests::SubmitResultRequest,
    types::{AssignmentState, Id},
};

use crate::{
    auth::Auth,
    result::{AppError, AppResult, ResultExt},
    state::AppState,
    util,
};

pub async fn submit_result(
    State(state): State<AppState>,
    Path(task_id): Path<Id<Task>>,
    Auth(user_id): Auth,
    Json(request): Json<SubmitResultRequest>,
) -> AppResult<(), SubmitResultError> {
    let mut tx = state.pool.begin().await?;

    let assignment = sqlx::query_as_unchecked!(
        Assignment,
        r#"
        SELECT
            *
        FROM
            assignments
        WHERE
            task_id = $1
            AND user_id = $2
        FOR UPDATE
        "#,
        task_id,
        user_id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_not_found(SubmitResultError::InvalidTask)?;

    if assignment.state == AssignmentState::Canceled {
        Err(AppError::Specific(SubmitResultError::AssignmentCanceled))?;
    }

    if assignment.state == AssignmentState::Expired {
        Err(AppError::Specific(SubmitResultError::AssignmentExpired))?;
    }

    sqlx::query_unchecked!(
        r#"
        INSERT INTO results (
            assignment_id,
            stdout,
            stderr,
            exit_code
        ) VALUES (
            $1,
            $2,
            $3,
            $4
        )
        "#,
        assignment.id,
        request.stdout,
        request.stderr,
        request.exit_code,
    )
    .execute(&mut *tx)
    .await
    .map_unique_violation(SubmitResultError::AlreadyExists)?;

    util::set_assignment_state(&[assignment.id], AssignmentState::Submitted)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
