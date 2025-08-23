use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{records::{Result}, types::Id};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateSubmitRequest {
    // First id is the assignment id that will change state, second is the "group id" it belongs with
    pub results: HashMap<Id<Result>, Option<Id<Result>>>,
}
