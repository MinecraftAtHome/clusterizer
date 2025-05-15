use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{records::Assignment, types::Id};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateSubmitRequest {
    pub assignments: HashMap<Id<Assignment>, Option<i32>>,
}
