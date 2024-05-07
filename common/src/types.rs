use serde::{Deserialize, Serialize};

pub type Id = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub id: Id,
}
