use serde::{Deserialize, Serialize};

use crate::{id::Id, types::Task, types::Result};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateOkRequest {
    pub task_id: Id<Task>,
    pub canonical_result_id: Id<Result>,
    pub result_ids: Vec<Id<Result>>
}
