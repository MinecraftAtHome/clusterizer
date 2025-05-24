use sqlx::{
    Postgres,
    postgres::{PgArguments, PgRow},
};

pub mod assignment_deadline;
pub mod select;
pub mod set_assignment_state;

pub use assignment_deadline::update_expired_assignments;
pub use select::Select;
pub use set_assignment_state::set_assignment_state;

type Query = sqlx::query::Query<'static, Postgres, PgArguments>;
type QueryScalar<T> = sqlx::query::QueryScalar<'static, Postgres, T, PgArguments>;
type Map<T> = sqlx::query::Map<'static, Postgres, fn(PgRow) -> sqlx::Result<T>, PgArguments>;
