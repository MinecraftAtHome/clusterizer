use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{records::record_impl, types::Id};

record_impl! {
    PATH = "platforms";

    Platform {
        id: Id<Platform>,
        created_at: DateTime<Utc>,
        name: String,
        tester_archive_url: String,
    }

    PlatformFilter {}
}
