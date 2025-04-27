use time::PrimitiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Result {
    pub id: i64,
    pub created_at: PrimitiveDateTime,
    pub assignment_id: i64,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}
