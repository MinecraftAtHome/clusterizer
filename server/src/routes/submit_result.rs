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

use crate::{auth::Auth, result::AppResult, state::AppState, util};

pub async fn submit_result(
    State(state): State<AppState>,
    Path(task_id): Path<Id<Task>>,
    Auth(user_id): Auth,
    Json(request): Json<SubmitResultRequest>,
) -> AppResult<(), SubmitResultError> {
    // TODO: fix race condition: assignment could get canceled before result is inserted

    let assignment_id = sqlx::query_scalar_unchecked!(
        r#"
        SELECT
            id "id: Id<Assignment>"
        FROM
            assignments
        WHERE
            task_id = $1
            AND user_id = $2
            AND state != 'canceled'
        "#,
        task_id,
        user_id,
    )
    .fetch_one(&state.pool)
    .await?;

    util::set_assignment_state(&state, AssignmentState::Submitted, &[assignment_id]).await?;

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
        assignment_id,
        request.stdout,
        request.stderr,
        request.exit_code,
    )
    .execute(&state.pool)
    .await?;

    Ok(())
}
