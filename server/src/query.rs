use clusterizer_common::{
    id::Id,
    types::{Assignment, Platform, Project, ProjectVersion, Result, Task, User},
};
use sqlx::{
    Postgres,
    postgres::{PgArguments, PgRow},
};

type Map<T> = sqlx::query::Map<'static, Postgres, fn(PgRow) -> sqlx::Result<T>, PgArguments>;

pub trait SelectAll: Sized {
    fn select_all() -> Map<Self>;
}

pub trait SelectAllBy<T>: Sized {
    fn select_all_by(id: Id<T>) -> Map<Self>;
}

pub trait SelectOne: Sized {
    fn select_one(id: Id<Self>) -> Map<Self>;
}

pub trait SelectOneBy<T>: Sized {
    fn select_one_by(id: Id<T>) -> Map<Self>;
}

macro_rules! select_all {
    ($t:ty: $query:literal) => {
        impl SelectAll for $t {
            fn select_all() -> Map<Self> {
                sqlx::query_as!(Self, $query)
            }
        }
    };
    ($t:ty: $query:literal, $u:ty) => {
        impl SelectAllBy<$u> for $t {
            fn select_all_by(id: Id<$u>) -> Map<Self> {
                sqlx::query_as!(Self, $query, id.raw())
            }
        }
    };
}

macro_rules! select_one {
    ($t:ty: $query:literal) => {
        impl SelectOne for $t {
            fn select_one(id: Id<Self>) -> Map<Self> {
                sqlx::query_as!(Self, $query, id.raw())
            }
        }
    };
    ($t:ty: $query:literal, $u:ty) => {
        impl SelectOneBy<$u> for $t {
            fn select_one_by(id: Id<$u>) -> Map<Self> {
                sqlx::query_as!(Self, $query, id.raw())
            }
        }
    };
}

select_all!(User: "SELECT * FROM users");
select_one!(User: "SELECT * FROM users WHERE id = $1");
select_all!(Project: "SELECT * FROM projects");
select_one!(Project: "SELECT * FROM projects WHERE id = $1");
select_all!(Platform: "SELECT * FROM platforms");
select_one!(Platform: "SELECT * FROM platforms WHERE id = $1");
select_all!(ProjectVersion: "SELECT * FROM project_versions");
select_one!(ProjectVersion: "SELECT * FROM project_versions WHERE id = $1");
select_all!(ProjectVersion: "SELECT * FROM project_versions WHERE project_id = $1", Project);
select_all!(ProjectVersion: "SELECT * FROM project_versions WHERE platform_id = $1", Platform);
select_all!(Task: "SELECT * FROM tasks");
select_one!(Task: "SELECT * FROM tasks WHERE id = $1");
select_all!(Task: "SELECT * FROM tasks WHERE project_id = $1", Project);
select_all!(Assignment: "SELECT * FROM assignments");
select_one!(Assignment: "SELECT * FROM assignments WHERE id = $1");
select_all!(Assignment: "SELECT * FROM assignments WHERE user_id = $1", User);
select_all!(Assignment: "SELECT * FROM assignments WHERE task_id = $1", Task);
select_all!(Result: "SELECT * FROM results");
select_one!(Result: "SELECT * FROM results WHERE id = $1");
select_one!(Result: "SELECT * FROM results WHERE assignment_id = $1", Assignment);
