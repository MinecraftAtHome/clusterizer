use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::Id;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct File {
    pub id: Id<File>,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub hash: Vec<u8>,
}

#[non_exhaustive]
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct FileFilter {}
