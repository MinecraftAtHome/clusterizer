use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct SubmitRequest {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub time_start: DateTime<Utc>,
    pub time_end: DateTime<Utc>,
}
