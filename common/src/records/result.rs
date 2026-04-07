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
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<Result>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<DateTime<Utc>>,
        "$3::int8[] IS NULL OR array_position($3, assignment_id) IS NOT NULL"
        assignment_id: Vec<Id<Assignment>>,
        "$4::text[] IS NULL OR array_position($4, stdout) IS NOT NULL"
        stdout: Vec<String>,
        "$5::text[] IS NULL OR array_position($5, stderr) IS NOT NULL"
        stderr: Vec<String>,
        "$6::int4[] IS NULL OR array_position($6, exit_code) IS NOT NULL"
        exit_code: Vec<Option<i32>>,
        "$7::int8[] IS NULL OR array_position($7, group_result_id) IS NOT NULL"
        group_result_id: Vec<Option<Id<Result>>>,
        "$8::result_state[] IS NULL OR array_position($8, state) IS NOT NULL"
        state: Vec<ResultState>,
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
