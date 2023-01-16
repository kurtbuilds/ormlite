use crate::error::{Error, Result};
use crate::query_builder::args::QueryBuilderArgs;
use crate::query_builder::{util, Placeholder};
use crate::model::Model;
use sqlmo::ToSql;

use sqlx::database::HasArguments;

use sqlx::{Executor, IntoArguments};
use std::marker::PhantomData;
use crate::join::JoinDescription;
use sqlmo::{Select, query::Where};

pub use sqlmo::query::Direction;
use sqlmo::query::{Join, SelectColumn};

// Add additional information to the sqlx::Database
pub trait DatabaseMetadata {
    fn dialect() -> sqlmo::Dialect;
    fn placeholder() -> Placeholder;
}

#[cfg(feature = "postgres")]
impl DatabaseMetadata for sqlx::postgres::Postgres {
    fn dialect() -> sqlmo::Dialect {
        sqlmo::Dialect::Postgres
    }

    fn placeholder() -> Placeholder {
        Placeholder::dollar_sign()
    }
}

#[cfg(feature = "sqlite")]
impl DatabaseMetadata for sqlx::sqlite::Sqlite {
    fn dialect() -> sqlmo::Dialect {
        sqlmo::Dialect::Sqlite
    }

    fn placeholder() -> Placeholder {
        Placeholder::question_mark()
    }
}

pub struct SelectQueryBuilder<'args, DB, Model>
where
    DB: sqlx::Database,
{
    pub query: Select,
    arguments: QueryBuilderArgs<'args, DB>,
    model: PhantomData<Model>,
    gen: Placeholder,
}

impl<'args, DB, M> SelectQueryBuilder<'args, DB, M>
where
    M: Sized + Send + Sync + Unpin + for<'r> sqlx::FromRow<'r, DB::Row> + 'static + Model<'static, DB>,
    DB: sqlx::Database + DatabaseMetadata,
    <DB as HasArguments<'args>>::Arguments: IntoArguments<'args, DB>,
{
    pub async fn fetch_all<'executor, E>(self, db: E) -> Result<Vec<M>>
    where
        E: Executor<'executor, Database = DB>,
    {
        let (text, args) = self.into_query_and_args()?;
        let z: &str = &text;
        util::query_as_with_recast_lifetime::<DB, M>(z, args)
            .fetch_all(db)
            .await
            .map_err(|e| Error::from(e))
    }

    pub async fn fetch_one<'executor, E>(self, db: E) -> Result<M>
    where
        E: Executor<'executor, Database = DB>,
    {
        let (text, args) = self.into_query_and_args()?;
        let z: &str = &text;
        util::query_as_with_recast_lifetime::<DB, M>(z, args)
            .fetch_one(db)
            .await
            .map_err(|e| Error::from(e))
    }

    pub async fn fetch_optional<'executor, E>(self, db: E) -> Result<Option<M>>
    where
        E: Executor<'executor, Database = DB>,
    {
        let (text, args) = self.into_query_and_args()?;
        let z: &str = &text;
        util::query_as_with_recast_lifetime::<DB, M>(z, args)
            .fetch_optional(db)
            .await
            .map_err(|e| Error::from(e))
    }

    pub fn with(mut self, name: &str, query: &str) -> Self {
        self.query = self.query.with_raw(name, query);
        self
    }

    /// Add a column to the query. Note you typically don't need this, as creating a query from
    /// `Model::select` will automatically add that model's columns.
    ///
    /// # Arguments
    /// * `column` - The column to add. Examples: "id", "name", "person.*"
    pub fn select(mut self, column: impl Into<String>) -> Self {
        self.query = self.query.select_raw(column.into());
        self
    }

    /// Add a WHERE clause to the query.
    /// Do not use format! to add parameters. Instead, use `?` as the placeholder, and add
    /// parameters with [`bind`](Self::bind).
    ///
    /// Postgres users: You can (and should) use `?` as the placeholder. You might not have a defined
    /// numerical order for your parameters, preventing $<N> syntax. Upon execution, the query
    /// builder replaces `?` with `$<N>`. If you need the same parameter multiple times, you should
    /// bind it multiple times. Arguments aren't moved, so this doesn't incur a memory cost. If you
    /// still want to re-use parameters, you can use $<N> placeholders. However, don't mix `?` and
    /// `$<N>` placeholders, as they will conflict.
    ///
    /// # Arguments
    /// * `clause` - The clause to add. Examples: "id = ?", "name = ?", "person.id = ?"
    pub fn where_(mut self, clause: &'static str) -> Self {
        self.query = self.query.where_raw(clause);
        self
    }

    /// Convenience method to add a `WHERE` and bind a value in one call.
    pub fn where_bind<T>(mut self, clause: &'static str, value: T) -> Self
        where
            T: 'args + Send + sqlx::Type<DB> + sqlx::Encode<'args, DB>,
    {
        self.query = self.query.where_raw(clause);
        self.arguments.add(value);
        self
    }
    /// Dangerous because it takes a string that could be user crafted. You should prefer `.where_` which
    /// takes a &'static str, and pass arguments with `.bind()`.
    pub fn dangerous_where(mut self, clause: &str) -> Self {
        self.query = self.query.where_raw(clause);
        self
    }

    pub fn join(mut self, join_description: JoinDescription) -> Self {
        self.query = self.query.join(join_description.to_join_clause(M::_table_name()));
        self.query.columns.extend(join_description.select_clause());
        self
    }

    #[doc(hidden)]
    #[deprecated(note = "Please use `where_` instead")]
    pub fn filter(self, clause: &'static str) -> Self {
        self.where_(clause)
    }

    /// Add a HAVING clause to the query.
    pub fn having(mut self, clause: &str) -> Self {
        self.query = self.query.having(Where::Raw(clause.to_string()));
        self
    }

    /// Add a GROUP BY clause to the query.
    ///
    /// # Arguments:
    /// * `clause`: The GROUP BY clause to add. Examples: "id", "id, date", "1, 2, ROLLUP(3)"
    pub fn group_by(mut self, clause: &str) -> Self {
        self.query = self.query.group_by(clause);
        self
    }

    /// Add an ORDER BY clause to the query.
    ///
    /// # Arguments:
    /// * `clause`: The ORDER BY clause to add. "created_at DESC", "id ASC NULLS FIRST"
    /// * `direction`: Direction::Asc or Direction::Desc
    pub fn order_by(mut self, clause: &str, direction: Direction) -> Self {
        self.query = self.query.order_by(clause, direction);
        self
    }

    pub fn order_asc(mut self, clause: &str) -> Self {
        self.query = self.query.order_asc(clause);
        self
    }

    pub fn order_desc(mut self, clause: &str) -> Self {
        self.query = self.query.order_desc(clause);
        self
    }

    /// Add a limit to the query.
    pub fn limit(mut self, limit: usize) -> Self {
        self.query = self.query.limit(limit);
        self
    }

    /// Add an offset to the query.
    pub fn offset(mut self, offset: usize) -> Self {
        self.query = self.query.offset(offset);
        self
    }

    /// Bind an argument to the query.
    pub fn bind<T>(mut self, value: T) -> Self
    where
        T: 'args + Send + sqlx::Type<DB> + sqlx::Encode<'args, DB>,
    {
        self.arguments.add(value);
        self
    }

    pub fn into_query_and_args(mut self) -> Result<(String, QueryBuilderArgs<'args, DB>)> {
        let q = self.query.to_sql(DB::dialect());
        let args = self.arguments;
        let (q, placeholder_count) = util::replace_placeholders(&q, &mut self.gen)?;
        if placeholder_count != args.len() {
            return Err(Error::OrmliteError(format!(
                "Failing to build query. {} placeholders were found in the query, but \
                {} arguments were provided.",
                placeholder_count,
                args.len(),
            )));
        }
        eprintln!("query: {}", q);
        Ok((q, args))
    }
}

impl<'args, DB: sqlx::Database + DatabaseMetadata, M: Model<'args, DB>> Default for SelectQueryBuilder<'args, DB, M> {
    fn default() -> Self {
        Self {
            query: Select::default().from(M::_table_name()),
            arguments: QueryBuilderArgs::default(),
            model: PhantomData,
            gen: DB::placeholder(),
        }
    }
}