use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub api_key: String,
}
