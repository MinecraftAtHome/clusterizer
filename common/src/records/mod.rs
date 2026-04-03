pub mod assignment;
pub mod file;
pub mod platform;
pub mod project;
pub mod project_version;
pub mod result;
pub mod task;
pub mod user;

pub use assignment::{Assignment, AssignmentFilter};
pub use file::{File, FileFilter};
pub use platform::{Platform, PlatformFilter};
pub use project::{Project, ProjectFilter};
pub use project_version::{ProjectVersion, ProjectVersionFilter};
pub use result::{Result, ResultFilter};
pub use task::{Task, TaskFilter};
pub use user::{User, UserFilter};

use crate::types::Id;

#[cfg(feature = "sqlx")]
mod sqlx {
    use sqlx::{
        Postgres,
        postgres::{PgArguments, PgRow},
    };

    pub type Map<T> =
        sqlx::query::Map<'static, Postgres, fn(PgRow) -> sqlx::Result<T>, PgArguments>;
}

pub trait Record: Sized {
    type Filter;

    const PATH: &str;

    #[cfg(feature = "sqlx")]
    fn select_all(filter: &Self::Filter) -> sqlx::Map<Self>;

    #[cfg(feature = "sqlx")]
    fn select_one(id: Id<Self>) -> sqlx::Map<Self>;
}

macro_rules! record_impl {
    (
        PATH = $path_literal:literal;

        $record_ident:ident {
            $($record_field_ident:ident: $record_field_ty:ty,)*
        }

        $filter_ident:ident {
            $(
                $filter_condition_literal:literal
                $filter_field_ident:ident: $filter_field_ty:ty,
            )*
        }
    ) => {
        #[derive(Clone, Hash, Debug, Serialize, Deserialize)]
        pub struct $record_ident {
            $(pub $record_field_ident: $record_field_ty,)*
        }

        #[non_exhaustive]
        #[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
        pub struct $filter_ident {
            $(pub $filter_field_ident: Option<$filter_field_ty>,)*
        }

        impl $filter_ident {
            $(
                pub fn $filter_field_ident(mut self, $filter_field_ident: impl Into<Option<$filter_field_ty>>) -> Self {
                    self.$filter_field_ident = $filter_field_ident.into();
                    self
                }
            )*
        }

        impl $crate::records::Record for $record_ident {
            type Filter = $filter_ident;

            const PATH: &str = $path_literal;

            #[cfg(feature = "sqlx")]
            fn select_all(#[allow(unused_variables)] filter: &Self::Filter) -> $crate::records::sqlx::Map<Self> {
                sqlx::query_as_unchecked!(
                    Self,
                    "SELECT * FROM " + $path_literal + " WHERE TRUE" $(+ " AND (" + $filter_condition_literal + ")")*,
                    $(filter.$filter_field_ident,)*
                )
            }

            #[cfg(feature = "sqlx")]
            fn select_one(id: $crate::types::Id<Self>) -> $crate::records::sqlx::Map<Self> {
                sqlx::query_as_unchecked!(Self, "SELECT * FROM " + $path_literal + " WHERE id = $1", id)
            }
        }
    };
}

use record_impl;
