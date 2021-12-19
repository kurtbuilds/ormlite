use std::marker::PhantomData;
use sqlx::Error;

type Args = String;

pub struct SelectQueryBuilder<T> {
    with: Vec<(String, String)>,
    pub(crate) select: Vec<String>,
    pub(crate) table: String,
    join: Vec<String>,
    filter: Vec<String>,
    group: Vec<String>,
    order: Vec<String>,
    having: Vec<String>,
    arguments: Vec<Args>,
    // _type: PhantomData<T>
}

impl<Model, DB> SelectQueryBuilder<Model>
    where DB: sqlx::Database + sqlx::database::HasStatementCache,
{
    pub fn with(mut self, name: &str, query: &str) -> Self {
        self.with.push((name.to_string(), query.to_string()));
        self
    }
    pub fn filter() { todo!() }
    pub fn join() { todo!() }
    pub fn having() { todo!() }
    pub fn group_by() { todo!() }
    pub fn order_by(mut self, clause: &str) { todo!() }

    pub fn bind(mut self, arg: &str) -> Self {
        self.arguments.push(arg.to_string());
        Self
    }

    pub async fn fetch_one<'e, 'c, E>(mut self, mut executor: E) -> Result<Model, Error>
        where E: sqlx::Executor<'c, Database=DB>,
    {
        let mut q = sqlx::query(&*self.build_query());
        for arg in &self.arguments {
            q = q.bind(arg);
        }
        executor.fetch_one(q).await
    }

    pub async fn fetch_all<'e, 'c, E>(mut self, executor: E) -> Result<Vec<Model>, Error>
        where E: sqlx::Executor<'c, Database=DB>,
    {
        // pub async fn fetch_all(mut self, mut db: &mut dyn sqlx::Connection) -> Result<Vec<Model>, Error> {
        let mut q = sqlx::query(&*self.build_query());
        for arg in &self.arguments {
            q = q.bind(arg);
        }
        executor.fetch_all(q).await
    }

    pub fn fetch_optional() {
        todo!()
    }

    pub fn fetch() {
        todo!()
    }

    pub fn execute() {
        todo!()
    }

    fn build_query(&self) -> String {
        let mut r = String::new();
        if self.with.len() {
            r += "WITH ";
            r += &self.with.map(|(name, clause)| format!("{name} as (\n{clause}\n)")).join(", ");
        }
        r += "SELECT ";
        r += &self.select.join(", ");
        r += &format!("\nFROM {from}", from = self.table);
        r += &self.join.join("\n");
        r += "\nWHERE 1=1\n";
        for clause in &self.filter {
            r += &format!("\nAND {clause}");
        }
        if self.group.len() {
            r += "\nGROUP BY ";
            r += &self.group.join(", ");
        }
        if self.order.len() {
            r += "\nORDER BY ";
            r += &self.order.join(", ");
        }
        if self.having.len() {
            r += "\nHAVING ";
            r += &self.having.join(", ");
        }
        r
    }
}


impl<T> Default for SelectQueryBuilder<T> {
    fn default() -> Self {
        Self {
            with: vec![],
            select: vec![],
            table: "".to_string(),
            join: vec![],
            filter: vec![],
            group: vec![],
            order: vec![],
            having: vec![],
            arguments: vec![],
        }
    }
}