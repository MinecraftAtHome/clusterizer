pub mod assignment;
pub mod file;
pub mod platform;
pub mod project;
pub mod project_runner;
pub mod result;
pub mod task;
pub mod user;

pub use assignment::{Assignment, AssignmentBuilder, AssignmentFilter};
pub use file::{File, FileBuilder, FileFilter};
pub use platform::{Platform, PlatformBuilder, PlatformFilter};
pub use project::{Project, ProjectBuilder, ProjectFilter};
pub use project_runner::{ProjectRunner, ProjectRunnerBuilder, ProjectRunnerFilter};
pub use result::{Result, ResultBuilder, ResultFilter};
pub use task::{Task, TaskBuilder, TaskFilter};
pub use user::{User, UserBuilder, UserFilter};

#[cfg(feature = "sqlx")]
mod sqlx {
    use sqlx::{
        Postgres,
        postgres::{PgArguments, PgRow},
    };

    pub type Query = sqlx::query::Query<'static, Postgres, PgArguments>;
    pub type QueryScalar<T> = sqlx::query::QueryScalar<'static, Postgres, T, PgArguments>;

    pub type Map<T> =
        sqlx::query::Map<'static, Postgres, fn(PgRow) -> sqlx::Result<T>, PgArguments>;
}

pub trait Record: Sized {
    type Filter;

    const PATH: &str;
}

#[cfg(feature = "reqwest")]
pub trait Get {
    type Ok: serde::de::DeserializeOwned;
    type Err: serde::de::DeserializeOwned;

    fn get(&self, client: &reqwest::Client, url: &str) -> reqwest::RequestBuilder;
}

#[cfg(feature = "sqlx")]
pub trait Select {
    type Record;

    fn select(&self) -> sqlx::Map<Self::Record>;
}

#[cfg(feature = "sqlx")]
pub trait Insert {
    type Record;

    fn insert(&self) -> sqlx::QueryScalar<crate::types::Id<Self::Record>>;
}

macro_rules! record_impl {
    (
        PATH = $table_name_literal:literal;

        $record_ident:ident {
            $($record_field_ident:ident: $record_field_ty:ty,)*
        }

        $filter_ident:ident {
            $(
                $filter_field_condition_literal:literal
                $filter_field_ident:ident: $filter_field_ty:ty,
            )*
        }

        $builder_ident:ident {
            $builder_first_field_name_literal:literal
            $builder_first_field_expression_literal:literal
            $builder_first_field_ident:ident: $builder_first_field_ty:ty,
            $(
                $builder_field_name_literal:literal
                $builder_field_expression_literal:literal
                $builder_field_ident:ident: $builder_field_ty:ty,
            )*
        }

        $update_ident:ident {
            $($update_fn_ident:ident($update_fn_name_literal:literal $update_fn_ty:ty);)*
        }
    ) => {
        #[cfg(feature = "sqlx")]
        pub trait $update_ident {
            $(fn $update_fn_ident(&self, value: $update_fn_ty) -> $crate::records::sqlx::Query;)*
        }

        #[derive(Clone, Hash, Debug, Serialize, Deserialize)]
        pub struct $record_ident {
            $(pub $record_field_ident: $record_field_ty,)*
        }

        #[non_exhaustive]
        #[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
        pub struct $filter_ident {
            $(pub $filter_field_ident: Option<$filter_field_ty>,)*
        }

        #[derive(Debug)]
        pub struct $builder_ident {
            pub $builder_first_field_ident: $builder_first_field_ty,
            $(pub $builder_field_ident: $builder_field_ty,)*
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

            const PATH: &str = $table_name_literal;
        }

        #[cfg(feature = "reqwest")]
        impl $crate::records::Get for $filter_ident {
            type Ok = Vec<$record_ident>;
            type Err = $crate::errors::Infallible;

            fn get(&self, client: &::reqwest::Client, url: &str) -> ::reqwest::RequestBuilder {
                let url = format!("{}/{}", url, $table_name_literal);
                let mut url = ::reqwest::Url::parse(&url).unwrap();
                let query = ::serde_qs::to_string(self).unwrap();
                url.set_query(Some(&query));
                client.get(url)
            }
        }

        #[cfg(feature = "reqwest")]
        impl $crate::records::Get for $crate::types::Id<$record_ident> {
            type Ok = $record_ident;
            type Err = $crate::errors::NotFound;

            fn get(&self, client: &::reqwest::Client, url: &str) -> ::reqwest::RequestBuilder {
                client.get(format!("{}/{}/{}", url, $table_name_literal, self))
            }
        }

        #[cfg(feature = "sqlx")]
        impl $crate::records::Select for $filter_ident {
            type Record = $record_ident;

            fn select(&self) -> $crate::records::sqlx::Map<Self::Record> {
                sqlx::query_as_unchecked!(
                    Self::Record,
                    "SELECT * FROM " + $table_name_literal + " WHERE TRUE" $(+ " AND (" + $filter_field_condition_literal + ")")*,
                    $(self.$filter_field_ident,)*
                )
            }
        }

        #[cfg(feature = "sqlx")]
        impl $crate::records::Select for $crate::types::Id<$record_ident> {
            type Record = $record_ident;

            fn select(&self) -> $crate::records::sqlx::Map<Self::Record> {
                sqlx::query_as_unchecked!(
                    Self::Record,
                    "SELECT * FROM " + $table_name_literal + " WHERE id = $1",
                    self
                )
            }
        }

        #[cfg(feature = "sqlx")]
        impl $crate::records::Select for [$crate::types::Id<$record_ident>] {
            type Record = $record_ident;

            fn select(&self) -> $crate::records::sqlx::Map<Self::Record> {
                sqlx::query_as_unchecked!(
                    Self::Record,
                    "SELECT * FROM " + $table_name_literal + " WHERE id = ANY($1)",
                    self
                )
            }
        }

        #[cfg(feature = "sqlx")]
        impl $crate::records::Insert for $builder_ident {
            type Record = $record_ident;

            fn insert(&self) -> $crate::records::sqlx::QueryScalar<$crate::types::Id<Self::Record>> {
                sqlx::query_scalar_unchecked!(
                    "INSERT INTO " + $table_name_literal + " (" + $builder_first_field_name_literal $(+ ", " + $builder_field_name_literal)* + ") VALUES (" + $builder_first_field_expression_literal $(+ ", " + $builder_field_expression_literal)* + ") RETURNING id \"id: _\"",
                    self.$builder_first_field_ident,
                    $(self.$builder_field_ident,)*
                )
            }
        }

        #[cfg(feature = "sqlx")]
        impl $update_ident for $crate::types::Id<$record_ident> {
            $(
                fn $update_fn_ident(&self, value: $update_fn_ty) -> $crate::records::sqlx::Query {
                    sqlx::query_unchecked!(
                        "UPDATE " + $table_name_literal + " SET " + $update_fn_name_literal + " = $2 WHERE id = $1",
                        self,
                        value,
                    )
                }
            )*
        }

        #[cfg(feature = "sqlx")]
        impl $update_ident for [$crate::types::Id<$record_ident>] {
            $(
                fn $update_fn_ident(&self, value: $update_fn_ty) -> $crate::records::sqlx::Query {
                    sqlx::query_unchecked!(
                        "UPDATE " + $table_name_literal + " SET " + $update_fn_name_literal + " = $2 WHERE id = ANY($1)",
                        self,
                        value,
                    )
                }
            )*
        }
    };
}

use record_impl;
