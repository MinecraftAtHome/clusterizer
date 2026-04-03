use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::LowerHex for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val: Vec<u8> = self.hash.clone();
        let mut str_builder: String = String::from("");
        for elem in val {
            str_builder.push_str(&format!("{:x}", elem));
        }
        write!(f, "{}", str_builder)
    }
}
