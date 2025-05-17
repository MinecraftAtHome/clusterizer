use serde::{Deserialize, Serialize};

use crate::{records::Assignment, types::Id};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateOkRequest {
    pub assignment_ids: Vec<Id<Assignment>>,
}
