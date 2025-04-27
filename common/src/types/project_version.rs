use time::PrimitiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub id: i64,
    pub created_at: PrimitiveDateTime,
    pub project_id: i64,
    pub platform_id: i64,
    pub archive_url: String,
}
