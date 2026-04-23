use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{records::record_impl, types::Id};

record_impl! {
    PATH = "files";

    File {
        id: Id<File>,
        created_at: DateTime<Utc>,
        url: String,
        hash: [u8; 32],
    }

    FileFilter {
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<File>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<DateTime<Utc>>,
        "$3::text[] IS NULL OR array_position($3, url) IS NOT NULL"
        url: Vec<String>,
        "$4::bytea[] IS NULL OR array_position($4, hash) IS NOT NULL"
        hash: Vec<[u8; 32]>,
    }

    FileBuilder {
        "url" "$1"
        url: String,
        "hash" "$2"
        hash: [u8; 32],
    }

    UpdateFile {}
}
