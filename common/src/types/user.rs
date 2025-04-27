use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub name: String,
}
