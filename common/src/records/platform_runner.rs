use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::Id;

use super::Platform;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct PlatformRunner {
    pub id: Id<PlatformRunner>,
    pub created_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub platform_id: Id<Platform>,
    pub archive_url: String,
}

#[non_exhaustive]
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct PlatformRunnerFilter {
    pub disabled: Option<bool>,
    pub platform_id: Option<Id<Platform>>,
}

impl PlatformRunnerFilter {
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn platform_id(mut self, platform_id: Id<Platform>) -> Self {
        self.platform_id = Some(platform_id);
        self
    }
}
