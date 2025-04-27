use time::PrimitiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub id: i64,
    pub created_at: PrimitiveDateTime,
    pub task_id: i64,
    pub user_id: i64,
    pub canceled: bool,
}
