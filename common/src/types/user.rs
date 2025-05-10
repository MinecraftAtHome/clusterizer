use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Id<User>,
    pub created_at: DateTime<Utc>,
    pub name: String,
}
