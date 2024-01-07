use futures::future::BoxFuture;
#[allow(unused_imports)]
use sqlmo::{Insert, Dialect, ToSql};
pub use sqlmo::query::OnConflict;
use crate::Result;

/// Represents an insert query.
/// We had to turn this into a model because we need to pass in the on_conflict configuration.
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
        self
    }
}

impl<'a, Acquire, Model: crate::model::Model<DB>, DB: sqlx::Database> std::future::IntoFuture for Insertion<'a, Acquire, Model, DB> {
    type Output = Result<Model>;
    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let q = self.insert.to_sql(Dialect::Postgres);
        (self.closure)(self.acquire, self.model, q)
    }
}
