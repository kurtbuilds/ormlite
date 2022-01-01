/// A model is a struct that represents a row in a relational database table.
use crate::Result;
use crate::SelectQueryBuilder;
use futures_core::future::BoxFuture;

/// `HasModelBuilder` structs are ones that can create `ModelBuilder`s.
/// The base model structs typically implement this.
pub trait HasModelBuilder<'a, ModelBuilder>
where
    ModelBuilder: Sized + Send,
    Self: Sized + Send,
{
    fn build() -> ModelBuilder;
    fn update_partial(&'a self) -> ModelBuilder;
}

/// A struct that is `Insertable` is expected to have same fields as the model, excluding fields
/// that have sane defaults at the database level. Concretely, if you have a Person struct:
/// #[derive(ormlite::Model)]
/// struct Person {
///     id: i32,
///     name: String,
///     age: i32,
/// }
///
/// Then the `Insertable` struct looks like:
/// struct InsertPerson {
///     name: String,
///     age: i32,
/// }
pub trait Insertable<'a, DB>
where
    Self: Sized + Send + Sync,
    DB: sqlx::Database,
{
    type Model;
    fn insert<'e: 'a, E>(self, db: E) -> BoxFuture<'a, Result<Self::Model>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>;
}

/// A struct that implements `ModelBuilder` implements the builder pattern for a model.
pub trait ModelBuilder<'a, DB>
where
    Self: Sized + Send + Sync,
    DB: sqlx::Database,
{
    type Model;

    fn insert<'e: 'a, E>(self, db: E) -> BoxFuture<'a, Result<Self::Model>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>;

    fn update<'e: 'a, E>(self, db: E) -> BoxFuture<'a, Result<Self::Model>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>;
}

/// The core trait. a struct that implements `Model` can also implement `HasModelBuilder`, `HasQueryBuilder` (and is required to implement `Insertable`)
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

pub trait HasQueryBuilder<DB, PlaceholderGenerator>
where
    DB: sqlx::Database,
    Self: Sized,
    PlaceholderGenerator: Iterator<Item = String>,
{
    fn select<'args>() -> SelectQueryBuilder<'args, DB, Self, PlaceholderGenerator>;
    // fn insert()
    // fn update()
    // fn delete()
}

pub trait TableMeta {
    fn table_name() -> &'static str;
    fn fields() -> &'static [&'static str];
    fn num_fields() -> usize;
    fn primary_key_column() -> &'static str;
}
