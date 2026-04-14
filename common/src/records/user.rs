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
        is_admin: bool,
    }

    UserFilter {
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<User>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<DateTime<Utc>>,
        "$3::timestamptz[] IS NULL OR array_position($3, disabled_at) IS NOT NULL"
        disabled_at: Vec<Option<DateTime<Utc>>>,
        "$4::text[] IS NULL OR array_position($4, name) IS NOT NULL"
        name: Vec<String>,
        "$5::bool[] IS NULL OR array_position($5, is_admin) IS NOT NULL"
        is_admin: bool,
    }

    UserBuilder {
        "name" "$1"
        name: String,
    }

    UpdateUser {}
}
