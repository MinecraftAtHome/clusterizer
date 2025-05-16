use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::Id;

use super::Assignment;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Result {
    pub id: Id<Result>,
    pub created_at: DateTime<Utc>,
    pub assignment_id: Id<Assignment>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}
