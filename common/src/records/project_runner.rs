use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{File, Platform, Project, record_impl},
    types::Id,
};

record_impl! {
    PATH = "project_runners";

    ProjectRunner {
        id: Id<ProjectRunner>,
        created_at: DateTime<Utc>,
        disabled_at: Option<DateTime<Utc>>,
        project_id: Id<Project>,
        platform_id: Id<Platform>,
        file_id: Id<File>,
    }

    ProjectRunnerFilter {
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<ProjectRunner>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<DateTime<Utc>>,
        "$3::timestamptz[] IS NULL OR array_position($3, disabled_at) IS NOT NULL"
        disabled_at: Vec<Option<DateTime<Utc>>>,
        "$4::int8[] IS NULL OR array_position($4, project_id) IS NOT NULL"
        project_id: Vec<Id<Project>>,
        "$5::int8[] IS NULL OR array_position($5, platform_id) IS NOT NULL"
        platform_id: Vec<Id<Platform>>,
        "$6::int8[] IS NULL OR array_position($6, file_id) IS NOT NULL"
        file_id: Vec<Id<File>>,
    }

    ProjectRunnerBuilder {
        "project_id" "$1"
        project_id: Id<Project>,
        "platform_id" "$2"
        platform_id: Id<Platform>,
        "file_id" "$3"
        file_id: Id<File>,
    }

    UpdateProjectRunner {}
}
