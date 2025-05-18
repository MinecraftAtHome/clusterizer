use clusterizer_common::{
    records::Assignment,
    types::{AssignmentState, Id},
};

use super::Query;

pub fn set_assignment_state(
    assignment_ids: &[Id<Assignment>],
    assignment_state: AssignmentState,
) -> Query {
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
}
