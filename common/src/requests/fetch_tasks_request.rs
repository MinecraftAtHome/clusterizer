use serde::{Deserialize, Serialize};

use crate::{id::Id, types::Project};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct FetchTasksRequest {
    pub project_ids: Vec<Id<Project>>,
}
