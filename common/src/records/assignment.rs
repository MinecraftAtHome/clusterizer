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
        "task_id = $1 IS NOT FALSE"
        task_id: Id<Task>,
        "user_id = $2 IS NOT FALSE"
        user_id: Id<User>,
        "state = $3 IS NOT FALSE"
        state: AssignmentState,
    }
}
