use serde::{Deserialize, Serialize};

use crate::{types::Id, records::Assignment};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateOkRequest {
    pub assignment_ids: Vec<Id<Assignment>>,
}
