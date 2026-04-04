use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{Assignment, record_impl},
    types::{Id, ResultState},
};

record_impl! {
    PATH = "results";

    Result {
        id: Id<Result>,
        created_at: DateTime<Utc>,
        assignment_id: Id<Assignment>,
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
        group_result_id: Option<Id<Result>>,
        state: ResultState,
    }

    ResultFilter {
        "assignment_id = $1 IS NOT FALSE"
        assignment_id: Id<Assignment>,
        "group_result_id = $2 OR $2 IS NULL"
        group_result_id: Id<Result>,
        "state = $3 IS NOT FALSE"
        state: ResultState,
    }

    ResultBuilder {
        "assignment_id" "$1"
        assignment_id: Id<Assignment>,
        "stdout" "$2"
        stdout: String,
        "stderr" "$3"
        stderr: String,
        "exit_code" "$4"
        exit_code: Option<i32>,
    }

    UpdateResult {
        update_group_result_id("group_result_id" Option<Id<Result>>);
        update_state("state" ResultState);
    }
}
