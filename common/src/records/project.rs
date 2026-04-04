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
        "created_by_user_id = $1 IS NOT FALSE"
        created_by_user_id: Id<User>,
        "disabled_at IS NULL IS DISTINCT FROM $2"
        disabled: bool,
    }

    ProjectBuilder {
        "created_by_user_id" "$1"
        created_by_user_id: Id<User>,
        "name" "$2"
        name: String,
    }

    UpdateProject {}
}
