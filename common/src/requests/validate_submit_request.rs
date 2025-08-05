use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{records::Assignment, types::Id};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateSubmitRequest {
    //First id is the assignment id that will change state, second is the "group id" it belongs with
    pub assignments: HashMap<Id<Assignment>, Id<Assignment>>,
}
