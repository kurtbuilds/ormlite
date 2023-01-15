use futures_core::future::BoxFuture;
use crate::Result;

#[derive(Debug, Copy, Clone)]
pub enum OnConflict {
    Ignore,
    Abort,
    // Replace,
}

/// Represents an insert query.
pub struct Insertion<'a, Acquire, Model, DB: sqlx::Database> {
    pub acquire: Acquire,
    pub model: Model,
    pub closure: Box<dyn 'static + Send + FnOnce(Acquire, Model, String) -> BoxFuture<'a, Result<Model>>>,
    pub table_name: &'static str,
    pub columns: &'static str,
    pub placeholders: &'static str,
    pub on_conflict: OnConflict,
    pub _db: std::marker::PhantomData<DB>,
}


impl<'a, Acquire, Model, DB: sqlx::Database> Insertion<'a, Acquire, Model, DB> {
    pub fn on_conflict(mut self, c: OnConflict) -> Self {
        self.on_conflict = c;
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
        let mut q = match self.on_conflict {
            OnConflict::Ignore => "INSERT OR IGNORE INTO ".to_string(),
            OnConflict::Abort => "INSERT OR ABORT INTO ".to_string(),
            // OnConflict::Replace => "INSERT OR REPLACE INTO ".to_string(),
        };
        q.push_str(&self.table_name);
        q.push_str(" (");
        q.push_str(&self.columns);
        q.push_str(") VALUES (");
        q.push_str(&self.placeholders);
        q.push_str(")");
        match self.on_conflict {
            OnConflict::Abort => q.push_str(" RETURNING *"),
            OnConflict::Ignore => {},
        }
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
        let mut q = "INSERT ".to_string();
        q.push_str(&self.table_name);
        q.push_str(" (");
        q.push_str(&self.columns);
        q.push_str(") VALUES (");
        q.push_str(&self.placeholders);
        q.push_str(")");
        match self.on_conflict {
            OnConflict::Ignore => q.push_str(" ON CONFLICT DO NOTHING "),
            OnConflict::Abort => {},
            // OnConflict::Replace => panic!
        }
        q.push_str(" RETURNING *");
        (self.closure)(self.acquire, self.model, q)
    }
}
