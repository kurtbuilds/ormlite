use crate::error::{Error, Result};
use crate::model::TableMeta;
use core::default::Default;
use sqlparser::dialect::GenericDialect;
use sqlparser::tokenizer::Tokenizer;
use sqlx::database::HasArguments;
use sqlx::query::QueryAs;
use sqlx::{query_as_with, Arguments, Database, Executor, IntoArguments};
use std::marker::PhantomData;

pub struct QueryBuilderArgs<'q, DB: HasArguments<'q>>(
    pub Box<<DB as HasArguments<'q>>::Arguments>,
    usize,
);

impl<'q, DB: Database> QueryBuilderArgs<'q, DB> {
    pub fn add<T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>>(&mut self, arg: T) {
        self.0.add(arg);
        self.1 += 1;
    }

    pub fn len(&self) -> usize {
        self.1
    }
}

impl<'q, DB: Database> IntoArguments<'q, DB> for QueryBuilderArgs<'q, DB> {
    fn into_arguments(self) -> <DB as HasArguments<'q>>::Arguments {
        *self.0
    }
}

fn replace_placeholders<T: Iterator<Item = String>>(
    sql: &str,
    placeholder_generator: &mut T,
) -> Result<(String, usize)> {
    let mut placeholder_count = 0usize;
    let dialect = GenericDialect {};
    // note this lib is inefficient because it's copying strings everywhere, instead
    // of using slices and an appropriate lifetime. probably want to swap out the lib at some point
    let tokens = Tokenizer::new(&dialect, sql).tokenize()?;
    let mut buf = String::with_capacity(sql.len() + 16);
    let mut it = tokens.iter();
    while let Some(tok) = it.next() {
        match tok {
            sqlparser::tokenizer::Token::Char(c) => match c {
                '?' => {
                    buf.push_str(&*placeholder_generator.next().unwrap());
                    placeholder_count += 1;
                }
                '$' => {
                    let next_tok = it.next();
                    if let Some(next_tok) = next_tok {
                        match next_tok {
                            sqlparser::tokenizer::Token::Number(text, _) => {
                                let n = text.parse::<usize>().map_err(|_| Error::OrmliteError(
                                    format!("Failed to parse number after a $ during query tokenization. Value was: {}",
                                        text
                                    )))?;
                                placeholder_count = std::cmp::max(placeholder_count, n);
                            }
                            _ => {}
                        }
                    }
                }
                _ => buf.push(*c),
            },
            _ => buf.push_str(&tok.to_string()),
        }
    }
    Ok((buf, placeholder_count))
}

pub struct SelectQueryBuilder<'args, DB, Model, PlaceholderGenerator>
where
    DB: sqlx::Database,
    PlaceholderGenerator: Iterator<Item = String>,
{
    with: Vec<(String, String)>,
    columns: Vec<String>,
    join: Vec<String>,
    wheres: Vec<String>,
    group: Vec<String>,
    order: Vec<String>,
    having: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,

    arguments: QueryBuilderArgs<'args, DB>,
    model: PhantomData<Model>,
    gen: PlaceholderGenerator,
}

fn query_as_with_recast_lifetime<'q, 'r, DB, Model>(
    s: &'q str,
    args: QueryBuilderArgs<'r, DB>,
) -> QueryAs<'q, DB, Model, QueryBuilderArgs<'q, DB>>
where
    'r: 'q,
    DB: Database,
    Model: for<'s> sqlx::FromRow<'s, DB::Row>,
    // <DB as HasArguments<'q>>::Arguments: IntoArguments<'q, DB>,
{
    // unsafe is safe b/c 'r: 'q. Rust isn't smart enough to know that downcasting of traits is safe, because when traits get lifetimes, it doesn't
    // know if the lifetime is covariant or contravariant, so it enforces equivalence. See: https://www.reddit.com/r/rust/comments/rox4j9/lifetime_inference_fails_when_lifetime_is_part_of/
    // But we know the trait is implemented by a struct, not a function, so we can do the downcast safely. Yay!
    let recast_args = unsafe { std::mem::transmute::<_, QueryBuilderArgs<'q, DB>>(args) };
    // unimplemented!()
    query_as_with(s, recast_args)
}

impl<'args, DB, Model, PlaceholderGenerator>
    SelectQueryBuilder<'args, DB, Model, PlaceholderGenerator>
where
    Model: Sized
        + Send
        + Sync
        + Unpin
        + TableMeta
        + for<'r> sqlx::FromRow<'r, DB::Row>
        + 'static
        + std::fmt::Debug,
    DB: sqlx::Database,
    <DB as HasArguments<'args>>::Arguments: IntoArguments<'args, DB>,
    PlaceholderGenerator: Iterator<Item = String>,
{
    pub async fn fetch_all<'executor, E>(mut self, db: E) -> Result<Vec<Model>>
    where
        E: Executor<'executor, Database = DB>,
    {
        let text = self.build_sql()?;
        let z: &str = &text;
        let args = std::mem::take(&mut self.arguments);
        query_as_with_recast_lifetime::<DB, Model>(z, args)
            .fetch_all(db)
            .await
            .map_err(|e| Error::from(e))
    }

    pub async fn fetch_one<'executor, E>(mut self, db: E) -> Result<Model>
    where
        E: Executor<'executor, Database = DB>,
    {
        let text = self.build_sql()?;
        let z: &str = &text;
        let args = std::mem::take(&mut self.arguments);
        query_as_with_recast_lifetime::<DB, Model>(z, args)
            .fetch_one(db)
            .await
            .map_err(|e| Error::from(e))
    }

    pub async fn fetch_optional<'executor, E>(mut self, db: E) -> Result<Option<Model>>
    where
        E: Executor<'executor, Database = DB>,
    {
        let text = self.build_sql()?;
        let z: &str = &text;
        let args = std::mem::take(&mut self.arguments);
        query_as_with_recast_lifetime::<DB, Model>(z, args)
            .fetch_optional(db)
            .await
            .map_err(|e| Error::from(e))
    }

    pub fn with(mut self, name: &str, query: &str) -> Self {
        self.with.push((name.to_string(), query.to_string()));
        self
    }

    /// Add a column to the query. Note you typically don't need this, as creating a query from
    /// `Model::select` will automatically add that model's columns.
    ///
    /// # Arguments
    /// * `column` - The column to add. Examples: "id", "name", "person.*"
    pub fn column(mut self, column: &str) -> Self {
        self.columns.push(column.to_string());
        self
    }

    /// Add a WHERE clause to the query.
    /// Do not use format! to add parameters. Instead, use `?` as the placeholder, and add
    /// parameters with [`bind`](Self::bind).
    ///
    /// Postgres users: You can (and should) use `?` as the placeholder. You might not have defined
    /// numerical ordinals for your parameters, preventing $<N> syntax. Upon execution, the query
    /// builder replaces `?` with `$<N>`. If you need the same parameter multiple times, you should
    /// bind it multiple times. Arguments aren't moved, so this doesn't incur a memory cost. If you
    /// still want to re-use parameters, you can use $<N> placeholders. However, don't mix `?` and
    /// `$<N>` placeholders, as they will conflict.
    ///
    /// # Arguments
    /// * `clause` - The clause to add. Examples: "id = ?", "name = ?", "person.id = ?"
    pub fn filter(mut self, clause: &str) -> Self {
        self.wheres.push(clause.to_string());
        self
    }

    /// Add a JOIN clause to the query.
    ///
    /// # Arguments:
    /// * `clause` - The join clause. If it doesn't start with any of `JOIN`, `INNER`,
    /// `LEFT`, `RIGHT`, `OUTER`, or `FULL` (case-insensitive), `JOIN` is assumed.
    pub fn join(mut self, clause: &str) -> Self {
        if let Some(x) = Some(clause.split_once(' ').map_or(clause, |x| x.0)) {
            if !vec!["join", "inner", "left", "right", "outer", "full"]
                .contains(&x.to_lowercase().as_str())
            {
                self.join.push("JOIN ".to_string() + clause);
                return self;
            }
        }
        self.join.push(clause.to_string());
        self
    }

    /// Add a HAVING clause to the query.
    pub fn having(mut self, clause: &str) -> Self {
        self.having.push(clause.to_string());
        self
    }

    /// Add a GROUP BY clause to the query.
    ///
    /// # Arguments:
    /// * `clause`: The GROUP BY clause to add. Examples: "id", "id, date", "1, 2, ROLLUP(3)"
    pub fn group_by(mut self, clause: &str) -> Self {
        self.group.push(clause.to_string());
        self
    }

    /// Add an ORDER BY clause to the query.
    ///
    /// # Arguments:
    /// * `clause`: The ORDER BY clause to add. "created_at DESC", "id ASC NULLS FIRST"
    pub fn order_by(mut self, clause: &str) -> Self {
        self.order.push(clause.to_string());
        self
    }

    /// Add a limit to the query.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add an offset to the query.
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
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

    fn build_sql(&mut self) -> Result<String> {
        let mut r = String::new();
        if !self.with.is_empty() {
            r += "WITH ";
            r += &self
                .with
                .iter()
                .map(|(name, clause)| format!("{} AS (\n{}\n)", name, clause))
                .collect::<Vec<_>>()
                .join(", ");
        }
        r += "SELECT\n";
        r += &self.columns.join(", ");
        r += &format!("\nFROM {}", Model::table_name());
        if !self.join.is_empty() {
            r += &self.join.join("\n");
        }
        if !self.wheres.is_empty() {
            r += "\nWHERE ";
            r += &*self
                .wheres
                .iter()
                .map(|clause| format!("({})", clause))
                .collect::<Vec<_>>()
                .join("\nAND ");
        }
        if !self.group.is_empty() {
            r += "\nGROUP BY ";
            r += &self.group.join("\n, ");
        }
        if !self.order.is_empty() {
            r += "\nORDER BY ";
            r += &self.order.join("\n, ");
        }
        if !self.having.is_empty() {
            r += "\nHAVING ";
            r += &self.having.join("\n, ");
        }
        if let Some(limit) = self.limit {
            r += &format!("\nLIMIT {}", limit);
            if let Some(offset) = self.offset {
                r += &format!(" OFFSET {}", offset);
            }
        }
        let (r, placeholder_count) = replace_placeholders(&r, &mut self.gen)?;
        if placeholder_count != self.arguments.len() {
            return Err(Error::OrmliteError(format!(
                "Failing to build query. {} placeholders were found in the query, but \
                {} arguments were provided.",
                placeholder_count,
                self.arguments.len(),
            )));
        }
        Ok(r)
    }
}

impl<'args, DB, Model> Default
    for SelectQueryBuilder<'args, DB, Model, Box<dyn Iterator<Item = String>>>
where
    DB: sqlx::Database,
{
    fn default() -> Self {
        Self {
            with: Vec::new(),
            columns: Vec::new(),
            join: Vec::new(),
            wheres: Vec::new(),
            group: Vec::new(),
            order: Vec::new(),
            having: Vec::new(),
            limit: None,
            offset: None,

            arguments: QueryBuilderArgs::default(),
            model: PhantomData,
            gen: Box::new(std::iter::repeat("?".to_string())),
        }
    }
}

impl<'q, DB: Database> Default for QueryBuilderArgs<'q, DB> {
    fn default() -> Self {
        Self(Box::new(<DB as HasArguments<'q>>::Arguments::default()), 0)
    }
}
