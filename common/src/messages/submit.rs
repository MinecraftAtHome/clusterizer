use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct SubmitRequest {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}
