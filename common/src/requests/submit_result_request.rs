use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct SubmitResultRequest {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}
