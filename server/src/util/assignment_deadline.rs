use crate::state::AppState;
use crate::util::set_assignment_state;
use clusterizer_common::records::Assignment;
use clusterizer_common::types::{AssignmentState, Id};
use sqlx::Postgres;
use sqlx::postgres::PgArguments;

pub fn fetch_assignments_id_exceed_deadline_for_update()
-> sqlx::query::QueryScalar<'static, Postgres, Id<Assignment>, PgArguments> {
    sqlx::query_scalar!(
        r#"
        SELECT
            id "id: Id<Assignment>"
        FROM
            assignments
        WHERE
            assignments.id IN (
                SELECT
                    assignments.id
                FROM
                    assignments
                LEFT JOIN
                    tasks on assignments.task_id = tasks.id
                WHERE
                    assignments.state = 'init'
                GROUP BY
                    tasks.id,
                    assignments.id
                HAVING (assignments.created_at + tasks.deadline) < NOW()
            )
        FOR SHARE NOWAIT
        "#
    )
}

pub async fn update_assignments_exceed_deadline(state: &AppState) -> sqlx::Result<usize> {
    let mut tx = state.pool.begin().await?;
    let ids = fetch_assignments_id_exceed_deadline_for_update()
        .fetch_all(&mut *tx)
        .await?;
    set_assignment_state(ids.as_slice(), AssignmentState::Expired)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(ids.len())
}
