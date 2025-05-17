use serde::{Deserialize, Serialize};

use crate::types::Assignment;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateErrRequest {
    pub assignments_needed: i32,
    pub assignments_error: Vec<Assignment>,
    pub assignments_inconclusive: Vec<Assignment>,
}
