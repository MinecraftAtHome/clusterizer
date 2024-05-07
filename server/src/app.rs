use crate::error::{pg_error, Error};
use tokio_postgres::{
    types::{FromSql, ToSql},
    Client, Row, ToStatement,
};

pub trait FromRow<'a> {
    fn from_row(row: &'a Row) -> Self;
}

pub struct App {
    client: Client,
}

impl App {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn query_one<T, U>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<U, Error>
    where
        T: ToStatement + ?Sized,
        for<'a> U: FromRow<'a>,
    {
        Ok(FromRow::from_row(
            &self
                .client
                .query_one(statement, params)
                .await
                .map_err(pg_error)?,
        ))
    }
}

impl<'a, T> FromRow<'a> for T
where
    T: FromSql<'a>,
{
    fn from_row(row: &'a Row) -> Self {
        row.get(0)
    }
}
