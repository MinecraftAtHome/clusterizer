use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{File, record_impl},
    types::Id,
};

record_impl! {
    PATH = "platforms";

    Platform {
        id: Id<Platform>,
        created_at: DateTime<Utc>,
        name: String,
        file_id: Id<File>,
    }

    PlatformFilter {
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<Platform>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<DateTime<Utc>>,
        "$3::text[] IS NULL OR array_position($3, name) IS NOT NULL"
        name: Vec<String>,
        "$4::int8[] IS NULL OR array_position($4, file_id) IS NOT NULL"
        file_id: Vec<Id<File>>,
    }

    PlatformBuilder {
        "name" "$1"
        name: String,
        "file_id" "$2"
        file_id: Id<File>,
    }

    UpdatePlatform {}
}
