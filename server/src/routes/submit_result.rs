use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    errors::SubmitResultError,
    records::{Assignment, Insert, ResultBuilder, Task},
    requests::SubmitResultRequest,
    types::{AssignmentState, Id},
};

use crate::{
    auth::Auth,
    result::{AppError, AppResult, ResultExt},
    state::AppState,
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

    ResultBuilder {
        assignment_id: assignment.id,
        stdout: request.stdout,
        stderr: request.stderr,
        exit_code: request.exit_code,
    }
    .insert()
    .fetch_one(&mut *tx)
    .await
    .map_unique_violation(SubmitResultError::AlreadyExists)?;

    tx.commit().await?;

    Ok(())
}
