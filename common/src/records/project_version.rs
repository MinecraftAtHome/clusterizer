use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{Platform, Project, record_impl},
    types::Id,
};

record_impl! {
    PATH = "project_versions";

    ProjectVersion {
        id: Id<ProjectVersion>,
        created_at: DateTime<Utc>,
        disabled_at: Option<DateTime<Utc>>,
        project_id: Id<Project>,
        platform_id: Id<Platform>,
        archive_url: String,
    }

    ProjectVersionFilter {
        "disabled_at IS NULL IS DISTINCT FROM $1"
        disabled: bool,
        "project_id = $2 IS NOT FALSE"
        project_id: Id<Project>,
        "platform_id = $3 IS NOT FALSE"
        platform_id: Id<Platform>,
    }
}
