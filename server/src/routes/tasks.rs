use axum::{
    Json,
    extract::{Path, State},
};
use clusterizer_common::{
    id::Id,
    messages::SubmitRequest,
    types::{Assignment, Platform, Task},
};

use crate::{auth::Auth, result::ApiResult, state::AppState};

pub async fn get_all(State(state): State<AppState>) -> ApiResult<Vec<Task>> {
    Ok(Json(
        sqlx::query_as_unchecked!(Task, "SELECT * FROM tasks")
            .fetch_all(&state.pool)
            .await?,
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(task_id): Path<Id<Task>>,
) -> ApiResult<Task> {
    Ok(Json(
        sqlx::query_as_unchecked!(Task, "SELECT * FROM tasks WHERE id = $1", task_id.raw())
            .fetch_one(&state.pool)
            .await?,
    ))
}

pub async fn fetch(
    State(state): State<AppState>,
    Path(platform_id): Path<Id<Platform>>,
    Auth(user_id): Auth,
) -> ApiResult<Vec<Task>> {
    let mut transaction = state.pool.begin().await?;
    let task = sqlx::query_as_unchecked!(
        Task,
        "
            WITH tasks_fetched AS (
                SELECT t.id
                FROM tasks t
                    JOIN projects p
                        ON t.project_id = p.id
                        AND p.active
                    JOIN project_versions pv
                        ON pv.project_id = p.id
                        AND pv.platform_id = $1
                WHERE 
                    t.assignments_needed > coalesce(array_length(t.assignment_user_ids, 1), 0)
                    AND NOT t.assignment_user_ids @> ARRAY[$2::bigint]
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE tasks
            SET 
                assignment_user_ids = assignment_user_ids || $2
            FROM tasks_fetched
                WHERE tasks.id = tasks_fetched.id
                RETURNING tasks.*;
        ",
        platform_id.raw(),
        user_id.raw()
    )
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(task) = task {
        sqlx::query!(
            "
            INSERT INTO assignments (
                task_id,
                user_id
            ) VALUES (
                $1,
                $2
            )
            ",
            task.id.raw(),
            user_id.raw()
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(Json(vec![task]))
    } else {
        sqlx::query_scalar!("SELECT 1 FROM platforms WHERE id = $1", platform_id.raw())
            .fetch_one(&mut *transaction)
            .await?;

        transaction.rollback().await?;

        Ok(Json(vec![]))
    }
}

pub async fn submit(
    State(state): State<AppState>,
    Path(task_id): Path<Id<Task>>,
    Auth(user_id): Auth,
    Json(request): Json<SubmitRequest>,
) -> ApiResult<()> {
    let assignment_id: Id<Assignment> = sqlx::query_scalar!(
        "
        SELECT
            id
        FROM
            assignments
        WHERE
            task_id = $1 AND
            user_id = $2 AND
            canceled_at IS NULL
        ",
        task_id.raw(),
        user_id.raw()
    )
    .fetch_one(&state.pool)
    .await?
    .into();

    sqlx::query!(
        "
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
        ",
        assignment_id.raw(),
        request.stdout,
        request.stderr,
        request.exit_code
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(()))
}
