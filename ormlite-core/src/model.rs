use crate::Result;
use crate::SelectQueryBuilder;
use futures_core::future::BoxFuture;

pub trait BuildsPartialModel<'a, PartialModel>
where
    PartialModel: Sized + Send,
    Self: Sized + Send,
{
    fn build() -> PartialModel;
    fn update_partial(&'a self) -> PartialModel;
}

//
// /** Declares that a struct can generate queries and execute them.
// Put another way, this trait is defined *on* the "builder" to execute it.
//  */
pub trait PartialModel<'a, DB>
where
    Self: Sized + Send + Sync,
    DB: sqlx::Database,
{
    type Model;

    fn insert<'db: 'a>(
        self,
        db: &'db mut <DB as sqlx::Database>::Connection,
    ) -> BoxFuture<'a, crate::Result<Self::Model>>;
    fn update<'db: 'a>(
        self,
        db: &'db mut <DB as sqlx::Database>::Connection,
    ) -> BoxFuture<'a, crate::Result<Self::Model>>;
}

pub trait Model<DB>
where
    DB: sqlx::Database,
    Self: Sized,
{
    fn insert<'e, E>(self, db: E) -> BoxFuture<'e, Result<Self>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>;

    fn update_all_fields<'e, E>(self, db: E) -> BoxFuture<'e, Result<Self>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>;

    fn delete<'e, E>(self, db: E) -> BoxFuture<'e, Result<()>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>;

    fn get_one<'e, 'a, Arg, E>(id: Arg, db: E) -> BoxFuture<'e, Result<Self>>
    where
        'a: 'e,
        E: 'e + sqlx::Executor<'e, Database = DB>,
        Arg: 'a + Send + for<'r> sqlx::Encode<'r, DB> + sqlx::Type<DB>;

    fn query(
        query: &str,
    ) -> sqlx::query::QueryAs<DB, Self, <DB as sqlx::database::HasArguments>::Arguments>;
}

pub trait BuildsQueryBuilder<DB, PlaceholderGenerator>
where
    DB: sqlx::Database,
    Self: Sized,
    PlaceholderGenerator: Iterator<Item = String>,
{
    fn select<'args>() -> SelectQueryBuilder<'args, DB, Self, PlaceholderGenerator>;
}

pub trait TableMeta {
    fn table_name() -> &'static str;
    fn fields() -> &'static [&'static str];
    fn num_fields() -> usize;
    fn primary_key_column() -> &'static str;
}
