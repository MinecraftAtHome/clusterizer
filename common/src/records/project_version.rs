use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{File, Platform, Project, record_impl},
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
        file_id: Id<File>,
    }

    ProjectVersionFilter {
        "disabled_at IS NULL IS DISTINCT FROM $1"
        disabled: bool,
        "project_id = $2 IS NOT FALSE"
        project_id: Id<Project>,
        "platform_id = $3 IS NOT FALSE"
        platform_id: Id<Platform>,
        "file_id = $4 IS NOT FALSE"
        file_id: Id<Platform>,
    }

    ProjectVersionBuilder {
        "project_id" "$1"
        project_id: Id<Project>,
        "platform_id" "$2"
        platform_id: Id<Platform>,
        "file_id" "$3"
        file_id: Id<File>,
    }

    UpdateProjectVersion {}
}
