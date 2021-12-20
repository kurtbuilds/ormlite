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
    fn insert(self, db: &mut <DB as sqlx::Database>::Connection) -> BoxFuture<crate::Result<Self>>;
    fn update_all_fields(
        self,
        db: &mut <DB as sqlx::Database>::Connection,
    ) -> BoxFuture<crate::Result<Self>>;
    fn delete(self, db: &mut <DB as sqlx::Database>::Connection) -> BoxFuture<crate::Result<()>>;
    fn get_one<'db, 'arg: 'db, T>(
        id: T,
        db: &'db mut <DB as sqlx::Database>::Connection,
    ) -> BoxFuture<'db, crate::Result<Self>>
    where
        T: 'arg + Send + for<'r> sqlx::Encode<'r, DB> + sqlx::Type<DB>;
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
