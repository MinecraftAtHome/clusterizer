use serde::{Deserialize, Serialize};

use crate::{id::Id, types::Result, types::Task};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateOkRequest {
    pub task_id: Id<Task>,
    pub result_ids: Vec<Id<Result>>,
}
