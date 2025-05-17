use serde::{Deserialize, Serialize};

use crate::{types::Id, records::Assignment};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateErrRequest {
    pub assignments_needed: i32,
    pub assignments_error: Vec<Id<Assignment>>,
    pub assignments_inconclusive: Vec<Id<Assignment>>,
}
