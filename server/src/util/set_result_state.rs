use clusterizer_common::{
    records::Result,
    types::{Id, ResultState},
};

use super::Query;

pub fn set_result_state(result_ids: &[Id<Result>], result_state: ResultState) -> Query {
    sqlx::query_unchecked!(
        r#"
        UPDATE 
            results
        SET 
            state = $1
        WHERE
            id = ANY($2)
        "#,
        result_state,
        result_ids,
    )
}
