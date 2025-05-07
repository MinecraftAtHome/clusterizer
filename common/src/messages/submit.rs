use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct SubmitRequest {
    pub task_id: i64,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct SubmitResponse;
