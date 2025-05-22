use crate::state::AppState;
use crate::util::{QueryScalar, set_assignment_state};
use clusterizer_common::records::Assignment;
use clusterizer_common::types::{AssignmentState, Id};

pub fn select_expired_assignment_ids() -> QueryScalar<Id<Assignment>> {
    sqlx::query_scalar!(
        r#"
        SELECT
            assignments.id "id: Id<Assignment>"
        FROM
            assignments
        WHERE
            assignments.state = 'init' AND deadline_at < NOW()
        FOR UPDATE
        "#
    )
}

pub async fn update_expired_assignments(state: &AppState) -> sqlx::Result<()> {
    let mut tx = state.pool.begin().await?;
    let ids = select_expired_assignment_ids().fetch_all(&mut *tx).await?;
    set_assignment_state(ids.as_slice(), AssignmentState::Expired)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    //TODO log rows changed
    Ok(())
}
