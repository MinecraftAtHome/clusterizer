use sqlx::{Postgres, postgres::PgArguments};

pub mod set_assignment_state;

pub use set_assignment_state::set_assignment_state;

type Query = sqlx::query::Query<'static, Postgres, PgArguments>;
