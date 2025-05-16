use serde::{Deserialize, Serialize};


#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateErrRequest {
    pub assignments_needed: i32,
}
