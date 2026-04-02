use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{File, Platform, record_impl},
    types::Id,
};

record_impl! {
    PATH = "platform_runners";

    PlatformRunner {
        id: Id<PlatformRunner>,
        created_at: DateTime<Utc>,
        platform_id: Id<Platform>,
        file_id: Id<File>,
    }

    PlatformRunnerFilter {
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<PlatformRunner>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<DateTime<Utc>>,
        "$3::int8[] IS NULL OR array_position($3, platform_id) IS NOT NULL"
        platform_id: Vec<Id<Platform>>,
        "$4::int8[] IS NULL OR array_position($4, file_id) IS NOT NULL"
        file_id: Vec<Id<File>>,
    }

    PlatformRunnerBuilder {
        "platform_id" "$1"
        platform_id: Id<Platform>,
        "file_id" "$2"
        file_id: Id<File>,
    }

    UpdatePlatformRunner {}
}
