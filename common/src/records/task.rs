use crate::types::Id;
use crate::types::duration::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Project, User};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Id<Task>,
    pub created_at: DateTime<Utc>,
    pub deadline: Duration,
    pub project_id: Id<Project>,
    pub stdin: String,
    pub assignments_needed: i32,
    pub assignment_user_ids: Vec<Id<User>>,
}

#[non_exhaustive]
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct TaskFilter {
    pub project_id: Option<Id<Project>>,
}

impl TaskFilter {
    pub fn project_id(mut self, project_id: Id<Project>) -> Self {
        self.project_id = Some(project_id);
        self
    }
}
