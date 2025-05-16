use serde::{Deserialize, Serialize};

use crate::{id::Id, types::Task};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ValidateErrRequest {
    pub assignments_needed: i32,
}
