use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub project_id: i64,
    pub platform_id: i64,
    pub archive_url: String,
}
