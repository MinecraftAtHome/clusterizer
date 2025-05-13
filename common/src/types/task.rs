use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;

use super::{Project, User};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Id<Task>,
    pub created_at: DateTime<Utc>,
    pub project_id: Id<Project>,
    pub stdin: String,
    pub assignments_needed: i32,
    pub assigned_to_userids: Vec<Id<User>>,
}
