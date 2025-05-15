use serde::{Deserialize, Serialize};

use crate::{id::Id, types::Task, types::Result};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateErrRequest {
    pub task_id: Id<Task>,
    pub assignments_needed: i32
}
