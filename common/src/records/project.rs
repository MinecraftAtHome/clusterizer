use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{records::User, types::Id};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Id<Project>,
    pub created_at: DateTime<Utc>,
    pub created_by_user_id: Id<User>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub name: String,
}

#[non_exhaustive]
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct ProjectFilter {
    pub created_by_user_id: Option<Id<User>>,
    pub disabled: Option<bool>,
}

impl ProjectFilter {
    pub fn created_by_user_id(mut self, created_by_user_id: Id<User>) -> Self {
        self.created_by_user_id = Some(created_by_user_id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }
}
