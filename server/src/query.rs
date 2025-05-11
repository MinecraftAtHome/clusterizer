use clusterizer_common::{
    id::Id,
    types::{Assignment, Platform, Project, ProjectVersion, Result, Task, User},
};
use sqlx::{
    Postgres,
    postgres::{PgArguments, PgRow},
};

type Map<T> = sqlx::query::Map<'static, Postgres, fn(PgRow) -> sqlx::Result<T>, PgArguments>;

pub trait QueryAll: Sized {
    fn query_all() -> Map<Self>;
}

pub trait QueryAllBy<T>: Sized {
    fn query_all_by(id: Id<T>) -> Map<Self>;
}

pub trait QueryOne: Sized {
    fn query_one(id: Id<Self>) -> Map<Self>;
}

pub trait QueryOneBy<T>: Sized {
    fn query_one_by(id: Id<T>) -> Map<Self>;
}

macro_rules! query_all {
    ($t:ty: $query:literal) => {
        impl QueryAll for $t {
            fn query_all() -> Map<Self> {
                sqlx::query_as!(Self, $query)
            }
        }
    };
    ($t:ty: $query:literal, $u:ty) => {
        impl QueryAllBy<$u> for $t {
            fn query_all_by(id: Id<$u>) -> Map<Self> {
                sqlx::query_as!(Self, $query, id.raw())
            }
        }
    };
}

macro_rules! query_one {
    ($t:ty: $query:literal) => {
        impl QueryOne for $t {
            fn query_one(id: Id<Self>) -> Map<Self> {
                sqlx::query_as!(Self, $query, id.raw())
            }
        }
    };
    ($t:ty: $query:literal, $u:ty) => {
        impl QueryOneBy<$u> for $t {
            fn query_one_by(id: Id<$u>) -> Map<Self> {
                sqlx::query_as!(Self, $query, id.raw())
            }
        }
    };
}

query_all!(User: "SELECT * FROM users");
query_one!(User: "SELECT * FROM users WHERE id = $1");
query_all!(Project: "SELECT * FROM projects");
query_one!(Project: "SELECT * FROM projects WHERE id = $1");
query_all!(Platform: "SELECT * FROM platforms");
query_one!(Platform: "SELECT * FROM platforms WHERE id = $1");
query_all!(ProjectVersion: "SELECT * FROM project_versions");
query_one!(ProjectVersion: "SELECT * FROM project_versions WHERE id = $1");
query_all!(ProjectVersion: "SELECT * FROM project_versions WHERE project_id = $1", Project);
query_all!(ProjectVersion: "SELECT * FROM project_versions WHERE platform_id = $1", Platform);
query_all!(Task: "SELECT * FROM tasks");
query_one!(Task: "SELECT * FROM tasks WHERE id = $1");
query_all!(Task: "SELECT * FROM tasks WHERE project_id = $1", Project);
query_all!(Assignment: "SELECT * FROM assignments");
query_one!(Assignment: "SELECT * FROM assignments WHERE id = $1");
query_all!(Assignment: "SELECT * FROM assignments WHERE user_id = $1", User);
query_all!(Assignment: "SELECT * FROM assignments WHERE task_id = $1", Task);
query_all!(Result: "SELECT * FROM results");
query_one!(Result: "SELECT * FROM results WHERE id = $1");
query_one!(Result: "SELECT * FROM results WHERE assignment_id = $1", Assignment);
