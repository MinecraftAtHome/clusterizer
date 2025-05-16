use serde::{Deserialize, Serialize};

use crate::{id::Id, types::Result};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateOkRequest {
    pub result_ids: Vec<Id<Result>>,
}
