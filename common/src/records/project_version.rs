use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::Id;

use super::{Platform, Project};

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub id: Id<ProjectVersion>,
    pub created_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub project_id: Id<Project>,
    pub platform_id: Id<Platform>,
    pub archive_url: String,
}

#[non_exhaustive]
#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub struct ProjectVersionFilter {
    pub disabled: Option<bool>,
    pub project_id: Option<Id<Project>>,
    pub platform_id: Option<Id<Platform>>,
}
