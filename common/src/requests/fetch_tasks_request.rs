use serde::{Deserialize, Serialize};

use crate::{records::Project, types::Id};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct FetchTasksRequest {
    pub project_ids: Vec<Id<Project>>,
    pub limit: usize,
}
