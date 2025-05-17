use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{AssignmentState, Id};

use super::{Task, User};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Id<Assignment>,
    pub created_at: DateTime<Utc>,
    pub task_id: Id<Task>,
    pub user_id: Id<User>,
    pub state: AssignmentState,
}

#[non_exhaustive]
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct AssignmentFilter {
    pub task_id: Option<Id<Task>>,
    pub user_id: Option<Id<User>>,
    pub state: Option<AssignmentState>,
}
