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
        "project_id = $1 IS NOT FALSE"
        project_id: Id<Project>,
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
