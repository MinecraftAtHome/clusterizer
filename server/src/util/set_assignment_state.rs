use clusterizer_common::{
    records::Assignment,
    types::{AssignmentState, Id},
};

use crate::state::AppState;

pub async fn set_assignment_state(
    state: &AppState,
    assignment_state: AssignmentState,
    assignment_ids: &[Id<Assignment>],
) -> sqlx::Result<()> {
    sqlx::query_unchecked!(
        r#"
        UPDATE 
            assignments
        SET 
            state = $1
        WHERE
            id = ANY($2)
        "#,
        assignment_state,
        assignment_ids
    )
    .execute(&state.pool)
    .await?;

    Ok(())
}
