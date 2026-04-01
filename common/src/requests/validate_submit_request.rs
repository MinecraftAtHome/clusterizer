use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{records::Result, types::Id};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateSubmitRequest {
    // Map from result id to group id. None means error.
    pub results: HashMap<Id<Result>, Option<Id<Result>>>,
}
