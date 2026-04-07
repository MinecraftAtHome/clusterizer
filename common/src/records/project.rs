use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    records::{User, record_impl},
    types::Id,
};

record_impl! {
    PATH = "projects";

    Project {
        id: Id<Project>,
        created_at: DateTime<Utc>,
        created_by_user_id: Id<User>,
        disabled_at: Option<DateTime<Utc>>,
        name: String,
    }

    ProjectFilter {
        "$1::int8[] IS NULL OR array_position($1, id) IS NOT NULL"
        id: Vec<Id<Project>>,
        "$2::timestamptz[] IS NULL OR array_position($2, created_at) IS NOT NULL"
        created_at: Vec<DateTime<Utc>>,
        "$3::int8[] IS NULL OR array_position($3, created_by_user_id) IS NOT NULL"
        created_by_user_id: Vec<Id<User>>,
        "$4::timestamptz[] IS NULL OR array_position($4, disabled_at) IS NOT NULL"
        disabled_at: Vec<Option<DateTime<Utc>>>,
        "$5::text[] IS NULL OR array_position($5, name) IS NOT NULL"
        name: Vec<String>,
    }

    ProjectBuilder {
        "created_by_user_id" "$1"
        created_by_user_id: Id<User>,
        "name" "$2"
        name: String,
    }

    UpdateProject {}
}
