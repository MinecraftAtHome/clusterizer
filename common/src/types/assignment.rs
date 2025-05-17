use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;

use super::{AssignmentState, Task, User};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Id<Assignment>,
    pub created_at: DateTime<Utc>,
    pub task_id: Id<Task>,
    pub user_id: Id<User>,
    pub state: AssignmentState,
}
