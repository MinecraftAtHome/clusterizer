use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Id<Project>,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub active: bool,
}
