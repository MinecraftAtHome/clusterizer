use crate::util::set_assignment_state;
use clusterizer_common::records::Assignment;
use clusterizer_common::types::{AssignmentState, Id};
use sqlx::postgres::PgArguments;
use sqlx::{PgPool, Postgres};

pub fn fetch_assignments_exceed_deadline()
-> sqlx::query::QueryScalar<'static, Postgres, Id<Assignment>, PgArguments> {
    sqlx::query_scalar!(
        r#"
        SELECT
            assignments.id "id: Id<Assignment>"
        FROM assignments
            LEFT JOIN tasks on assignments.task_id = tasks.id
        WHERE assignments.state = 'init'
        GROUP BY tasks.id, assignments.id
        HAVING (assignments.created_at + tasks.deadline) < NOW()
        "#
    )
}

pub async fn update_assignments_exceed_deadline(pool: &PgPool) -> sqlx::Result<usize> {
    let ids = fetch_assignments_exceed_deadline().fetch_all(pool).await?;
    set_assignment_state(ids.as_slice(), AssignmentState::Expired)
        .execute(pool)
        .await?;
    Ok(ids.len())
}
