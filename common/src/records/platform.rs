use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{records::File, types::Id};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Platform {
    pub id: Id<Platform>,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub file_id: Id<File>,
}

#[non_exhaustive]
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct PlatformFilter {
    pub file_id: Option<Id<File>>,
}

impl PlatformFilter {
    pub fn file_id(mut self, file_id: Id<File>) -> Self {
        self.file_id = Some(file_id);
        self
    }
}
