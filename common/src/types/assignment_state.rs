use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "assignment_state", rename_all = "snake_case")
)]
pub enum AssignmentState {
    Init,
    Canceled,
    Expired,
    Submitted,
    Valid,
    Invalid,
    Inconclusive,
    Error,
}
