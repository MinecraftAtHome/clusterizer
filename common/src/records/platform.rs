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
        "file_id = $1 IS NOT FALSE"
        file_id: Id<File>,
    }

    PlatformBuilder {
        "name" "$1"
        name: String,
        "file_id" "$2"
        file_id: Id<File>,
    }

    UpdatePlatform {}
}
