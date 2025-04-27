use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub project_id: i64,
    pub stdin: String,
}
