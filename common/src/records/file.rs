use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{records::record_impl, types::Id};

record_impl! {
    PATH = "files";

    File {
        id: Id<File>,
        created_at: DateTime<Utc>,
        url: String,
        hash: Vec<u8>,
    }

    FileFilter {}

    FileBuilder {
        "url" "$1"
        url: String,
        "hash" "$2"
        hash: Vec<u8>,
    }

    UpdateFile {}
}
