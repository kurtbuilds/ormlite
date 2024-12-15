use crate::CoreResult;
use futures::future::BoxFuture;
pub use sqlmo::query::OnConflict;
use sqlmo::{Cte, Dialect, Insert, Select, SelectColumn, ToSql, Union};

/// Represents an insert query.
/// We had to turn this into a model because we need to pass in the on_conflict configuration.
pub struct Insertion<'a, Acquire, Model, DB: sqlx::Database> {
    pub acquire: Acquire,
    pub model: Model,
    pub closure: Box<dyn 'static + Send + FnOnce(Acquire, Model, String) -> BoxFuture<'a, CoreResult<Model>>>,
    pub insert: Insert,
    pub _db: std::marker::PhantomData<DB>,
}

impl<'a, Acquire, Model, DB: sqlx::Database> Insertion<'a, Acquire, Model, DB> {
    pub fn on_conflict(mut self, c: OnConflict) -> Self {
        self.insert.on_conflict = c;
        self
    }
}

impl<'a, Acquire, Model: crate::model::Model<DB>, DB: sqlx::Database> std::future::IntoFuture
    for Insertion<'a, Acquire, Model, DB>
{
    type Output = CoreResult<Model>;
    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        // hack to get around the fact that postgres drops the return
        // value in ON CONFLICT DO NOTHING case
        let q = if matches!(self.insert.on_conflict, OnConflict::Ignore) {
            let insert_as_select = Select {
                ctes: vec![Cte::new("inserted", self.insert)],
                columns: vec![SelectColumn::raw("*")],
                from: Some("inserted".into()),
                ..Select::default()
            };
            let pkey = Model::primary_key().unwrap();
            let plc_idx = Model::primary_key_placeholder_idx().unwrap();
            let select_existing = Select {
                from: Some(Model::table_name().into()),
                columns: Model::table_columns().iter().map(|&c| c.into()).collect(),
                where_: format!("{pkey} = ${plc_idx}").into(),
                ..Select::default()
            };
            let union = Union {
                all: true,
                queries: vec![insert_as_select, select_existing],
            };
            union.to_sql(Dialect::Postgres)
        } else {
            self.insert.to_sql(Dialect::Postgres)
        };
        (self.closure)(self.acquire, self.model, q)
    }
}
