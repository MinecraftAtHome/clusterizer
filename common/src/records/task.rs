use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{Id, Interval};

use super::{Project, Result, User};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Id<Task>,
    pub created_at: DateTime<Utc>,
    pub deadline: Interval,
    pub project_id: Id<Project>,
    pub stdin: String,
    pub assignments_needed: i32,
    pub assignment_user_ids: Vec<Id<User>>,
    pub canonical_result_id: Option<Id<Result>>,
    pub quorum: i32,
}

#[non_exhaustive]
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct TaskFilter {
    pub project_id: Option<Id<Project>>,
    pub canonical_result_id: Option<Id<Result>>,
}

impl TaskFilter {
    pub fn project_id(mut self, project_id: Id<Project>) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn canonical_result_id(mut self, canonical_result_id: Id<Result>) -> Self {
        self.canonical_result_id = Some(canonical_result_id);
        self
    }
}
