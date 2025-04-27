use time::PrimitiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub created_at: PrimitiveDateTime,
    pub name: String,
    pub active: bool,
}
