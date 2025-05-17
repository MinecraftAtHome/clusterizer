use serde::{Deserialize, Serialize};

use crate::{id::Id, types::Assignment};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateOkRequest {
    pub assignment_ids: Vec<Id<Assignment>>,
}
