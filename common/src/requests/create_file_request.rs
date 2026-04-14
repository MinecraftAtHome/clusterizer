use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct CreateFileRequest {
    pub url: String,
    pub hash: Vec<u8>,
}
