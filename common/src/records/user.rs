use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{records::record_impl, types::Id};

record_impl! {
    PATH = "users";

    User {
        id: Id<User>,
        created_at: DateTime<Utc>,
        disabled_at: Option<DateTime<Utc>>,
        name: String,
    }

    UserFilter {
        "disabled_at IS NULL IS DISTINCT FROM $1"
        disabled: bool,
    }
}
