use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{AssignmentState, Id};

use super::{Task, User};

#[derive(Clone, Copy, Hash, Debug, Serialize, Deserialize)]
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

impl AssignmentFilter {
    pub fn task_id(mut self, task_id: Id<Task>) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn user_id(mut self, user_id: Id<User>) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn state(mut self, state: AssignmentState) -> Self {
        self.state = Some(state);
        self
    }
}
