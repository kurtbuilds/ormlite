use futures_core::future::BoxFuture;
#[allow(unused_imports)]
use sqlmo::{Insert, Dialect, ToSql};
pub use sqlmo::query::OnConflict;
use crate::Result;

/// Represents an insert query.
pub struct Insertion<'a, Acquire, Model, DB: sqlx::Database> {
    pub acquire: Acquire,
    pub model: Model,
    pub closure: Box<dyn 'static + Send + FnOnce(Acquire, Model, String) -> BoxFuture<'a, Result<Model>>>,
    pub insert: Insert,
    pub _db: std::marker::PhantomData<DB>,
}


impl<'a, Acquire, Model, DB: sqlx::Database> Insertion<'a, Acquire, Model, DB> {
    pub fn on_conflict(mut self, c: OnConflict) -> Self {
        self.insert.on_conflict = c;
        if c == OnConflict::Ignore {
            self.insert.returning = Vec::new();
        }
        self
    }
}


#[cfg(feature = "sqlite")]
impl<'a, Acquire, Model> std::future::IntoFuture for Insertion<'a, Acquire, Model, sqlx::sqlite::Sqlite>
    where
        Model: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
        Acquire: Send,
{
    type Output = Result<Model>;
    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let q = self.insert.to_sql(Dialect::Sqlite);
        (self.closure)(self.acquire, self.model, q)
    }
}

#[cfg(feature = "postgres")]
impl<'a, Acquire, Model> std::future::IntoFuture for Insertion<'a, Acquire, Model, sqlx::postgres::Postgres>
    where
        Model: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin
{
    type Output = Result<Model>;
    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let q = self.insert.to_sql(Dialect::Postgres);
        (self.closure)(self.acquire, self.model, q)
    }
}
