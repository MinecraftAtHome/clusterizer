use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{Project, User, record_impl},
    types::{Id, Interval},
};

record_impl! {
    PATH = "tasks";

    Task {
        id: Id<Task>,
        created_at: DateTime<Utc>,
        deadline: Interval,
        project_id: Id<Project>,
        stdin: String,
        assignments_needed: i32,
        assignment_user_ids: Vec<Id<User>>,
        quorum: i32,
    }

    TaskFilter {
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<Task>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<DateTime<Utc>>,
        "$3::interval[] IS NULL OR array_position($3, deadline) IS NOT NULL"
        deadline: Vec<Interval>,
        "$4::int8[] IS NULL OR array_position($4, project_id) IS NOT NULL"
        project_id: Vec<Id<Project>>,
        "$5::text[] IS NULL OR array_position($5, stdin) IS NOT NULL"
        stdin: Vec<String>,
        "$6::int4[] IS NULL OR array_position($6, assignments_needed) IS NOT NULL"
        assignments_needed: Vec<i32>,
        "$7::int8[] IS NULL OR array_position($7, quorum) IS NOT NULL"
        quorum: Vec<i32>,
    }

    TaskBuilder {
        "deadline" "$1"
        deadline: Interval,
        "project_id" "$2"
        project_id: Id<Project>,
        "stdin" "$3"
        stdin: String,
        "quorum" "$4"
        quorum: i32,
    }

    UpdateTask {
        update_assignments_needed("assignments_needed" i32);
    }
}
