use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{Task, User, record_impl},
    types::{AssignmentState, Id},
};

record_impl! {
    PATH = "assignments";

    Assignment {
        id: Id<Assignment>,
        created_at: DateTime<Utc>,
        deadline_at: DateTime<Utc>,
        task_id: Id<Task>,
        user_id: Id<User>,
        state: AssignmentState,
    }

    AssignmentFilter {
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<Task>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<Id<User>>,
        "$3::timestamptz[] IS NULL OR array_position($3, deadline_at) IS NOT NULL"
        deadline_at: Vec<AssignmentState>,
        "$4::int8[] IS NULL OR array_position($4, task_id) IS NOT NULL"
        task_id: Vec<Id<Task>>,
        "$5::int8[] IS NULL OR array_position($5, user_id) IS NOT NULL"
        user_id: Vec<Id<User>>,
        "$6::assignment_state[] IS NULL OR array_position($6, state) IS NOT NULL"
        state: Vec<AssignmentState>,
    }

    AssignmentBuilder {
        "task_id" "$1"
        task_id: Id<Task>,
        "user_id" "$2"
        user_id: Id<User>,
    }

    UpdateAssignment {
        update_state("state" AssignmentState);
    }
}
