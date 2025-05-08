use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub task_id: i64,
    pub user_id: i64,
    pub canceled_at: Option<DateTime<Utc>>,
}
