/// A model is a struct that represents a row in a relational database table.
/// Using the `[derive(ormlite::Model)]` macro, it will acquire the following traits:
///
///  - `ormlite::Model`, giving it direct database access, e.g. `insert`, `update_all_fields`, etc.
///  - `ormlite::HasModelBuilder`, letting it build partials, so you can insert or update some
///    fields instead of all of them at once, e.g. `model.name("John").update()`
///  - `ormlite::TableMeta`, which you typically don't use directly, but provides table metadata
///    (e.g. table name)
///
use crate::Result;
use crate::SelectQueryBuilder;
use futures::future::BoxFuture;

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
pub trait Insertable<DB>
    where
        Self: Sized + Send + Sync,
        DB: sqlx::Database,
{
    type Model;
    fn insert<'e, E>(self, db: E) -> BoxFuture<'e, Result<Self::Model>>
        where
            E: 'e + sqlx::Executor<'e, Database=DB>;
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
            E: 'e + sqlx::Executor<'e, Database=DB>;

    fn update<'e: 'a, E>(self, db: E) -> BoxFuture<'a, Result<Self::Model>>
        where
            E: 'e + sqlx::Executor<'e, Database=DB>;

    /// Build the model, don't insert or update it.
    fn build(self) -> Self::Model;
}

/// The core trait. a struct that implements `Model` can also implement `HasModelBuilder`, (and is required to implement `Insertable`)
pub trait Model<'slf, DB>
    where
        DB: sqlx::Database,
        Self: Sized,
{
    type ModelBuilder: ModelBuilder<'slf, DB>;

    /// Create a builder-pattern object to update one or more columns.
    /// You can also use `update_all_fields` to update all columns.
    fn update_partial(&'slf self) -> Self::ModelBuilder;

    #[doc(hidden)]
    fn _table_name() -> &'static str;

    /// All table columns. Excludes relation fields.
    #[doc(hidden)]
    fn _table_columns() -> &'static [&'static str];

    /// Insert the model into the database.
    fn insert<'a, A>(self, conn: A) -> crate::insert::Insertion<'a, A, Self, DB>
        where
            A: 'a + Send + sqlx::Acquire<'a, Database=DB>,
            Self: Send;

    /// `Model` objects can't track what fields are updated, so this method will update all fields.
    /// If you want to update only some fields, use `update_partial` instead.
    fn update_all_fields<'e, E>(self, db: E) -> BoxFuture<'e, Result<Self>>
        where
            E: 'e + sqlx::Executor<'e, Database=DB>;

    fn delete<'e, E>(self, db: E) -> BoxFuture<'e, Result<()>>
        where
            E: 'e + sqlx::Executor<'e, Database=DB>;

    /// Get by primary key.
    fn fetch_one<'e, 'a, Arg, E>(id: Arg, db: E) -> BoxFuture<'e, Result<Self>>
        where
            'a: 'e,
            E: 'e + sqlx::Executor<'e, Database=DB>,
            Arg: 'a + Send + sqlx::Encode<'a, DB> + sqlx::Type<DB>;

    /// If query building isn't meeting your needs, use this method to query the table using raw SQL.
    fn query(
        query: &str,
    ) -> sqlx::query::QueryAs<DB, Self, <DB as sqlx::database::HasArguments>::Arguments>;

    /// Create a `SelectQueryBuilder` to build a query.
    fn select<'args>() -> SelectQueryBuilder<'args, DB, Self>;

    fn builder() -> Self::ModelBuilder;
}