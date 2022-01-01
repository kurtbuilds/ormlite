use ormlite_core::model::{
    HasInsertModel, HasQueryBuilder, Insertable, Model, ModelBuilder, TableMeta,
};
use ormlite_core::{BoxFuture, Error, Result, SelectQueryBuilder};

pub static PLACEHOLDER: &str = "?";
pub static CREATE_TABLE_SQL: &str =
    "CREATE TABLE person (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)";

type DB = sqlx::Sqlite;

#[derive(sqlx::FromRow, Debug)]
pub struct Person {
    pub id: u32,
    pub name: String,
    pub age: u8,
}

impl ormlite_core::model::TableMeta for Person {
    fn table_name() -> &'static str {
        "person"
    }

    fn fields() -> &'static [&'static str] {
        &["id", "name", "age"]
    }

    fn num_fields() -> usize {
        3
    }
    fn primary_key_column() -> &'static str {
        "id"
    }
}

impl crate::model::Model<DB> for Person {
    fn insert<'e, E>(self, db: E) -> BoxFuture<'e, Result<Self>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>,
    {
        Box::pin(async move {
            let q = format!(
                "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                Self::table_name(),
                Self::fields().join(", "),
                (0..Self::num_fields())
                    .map(|_| PLACEHOLDER)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            sqlx::query_as::<_, Self>(&q)
                .bind(self.id)
                .bind(self.name)
                .bind(self.age)
                .fetch_one(db)
                .await
                .map_err(Error::from)
        })
    }
    fn update_all_fields<'e, E>(self, db: E) -> BoxFuture<'e, Result<Self>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>,
    {
        Box::pin(async move {
            let q = format!(
                "UPDATE {} SET {} WHERE {} = {} RETURNING *",
                Self::table_name(),
                "name=?, age=?",
                Self::primary_key_column(),
                PLACEHOLDER,
            );
            sqlx::query_as::<_, Self>(&q)
                .bind(self.name)
                .bind(self.age)
                .bind(self.id)
                .fetch_one(db)
                .await
                .map_err(Error::from)
        })
    }

    fn delete<'e, E>(self, db: E) -> BoxFuture<'e, Result<()>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>,
    {
        Box::pin(async move {
            let q = format!(
                "DELETE from {} WHERE {} = {}",
                Self::table_name(),
                Self::primary_key_column(),
                PLACEHOLDER,
            );
            let row = sqlx::query(&q).bind(self.id).execute(db).await?;
            if row.rows_affected() == 0 {
                Err(sqlx::Error::RowNotFound.into())
            } else {
                Ok(())
            }
        })
    }

    fn get_one<'e, 'a, Arg, E>(id: Arg, db: E) -> BoxFuture<'e, Result<Self>>
    where
        'a: 'e,
        E: 'e + sqlx::Executor<'e, Database = DB>,
        Arg: 'a + Send + for<'r> sqlx::Encode<'r, DB> + sqlx::Type<DB>,
    {
        let text = format!(
            "SELECT * FROM {} WHERE {} = {}",
            Self::table_name(),
            Self::primary_key_column(),
            PLACEHOLDER,
        );
        Box::pin(async move {
            sqlx::query_as::<DB, Self>(&text)
                .bind(id)
                .fetch_one(db) // executor outlives the
                .await
                .map_err(Error::from)
        })
    }

    fn query(
        query: &str,
    ) -> sqlx::query::QueryAs<DB, Person, <DB as ::sqlx::database::HasArguments>::Arguments> {
        sqlx::query_as::<_, Self>(query)
    }
}

impl HasQueryBuilder<DB, Box<dyn Iterator<Item = String>>> for Person {
    fn select<'a>() -> SelectQueryBuilder<'a, DB, Self, Box<dyn Iterator<Item = String>>> {
        SelectQueryBuilder::default().column(&format!("{}.*", Self::table_name()))
    }
}

// done
impl<'a> ormlite_core::model::HasModelBuilder<'a, PartialPerson<'a>> for Person {
    fn build() -> PartialPerson<'a> {
        PartialPerson::default()
    }

    fn update_partial(&'a self) -> PartialPerson<'a> {
        let mut partial = PartialPerson::default();
        partial.updating = Some(self);
        partial
    }
}

pub struct PartialPerson<'a> {
    id: Option<u32>,
    name: Option<String>,
    age: Option<u8>,

    updating: Option<&'a Person>,
}

impl<'a> Default for PartialPerson<'a> {
    fn default() -> Self {
        PartialPerson {
            id: None,
            name: None,
            age: None,
            updating: None,
        }
    }
}

impl<'a> PartialPerson<'a> {
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn age(mut self, age: u8) -> Self {
        self.age = Some(age);
        self
    }

    fn modified_fields(&self) -> Vec<&'static str> {
        let mut ret = Vec::new();
        if self.id.is_some() {
            ret.push("id");
        }
        if self.name.is_some() {
            ret.push("name");
        }
        if self.age.is_some() {
            ret.push("age");
        }
        ret
    }
}

impl<'a> ModelBuilder<'a, DB> for PartialPerson<'a> {
    type Model = Person;

    fn insert<'e: 'a, E>(self, db: E) -> BoxFuture<'a, Result<Self::Model>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>,
    {
        Box::pin(async move {
            let insert_fields = self.modified_fields();
            let query = format!(
                "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                Self::Model::table_name(),
                insert_fields.join(", "),
                insert_fields
                    .iter()
                    .map(|_| PLACEHOLDER)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            let mut q = sqlx::query_as::<DB, Self::Model>(&query);
            if let Some(value) = self.id {
                q = q.bind(value);
            }
            if let Some(value) = self.name {
                q = q.bind(value);
            }
            if let Some(value) = self.age {
                q = q.bind(value);
            }
            q.fetch_one(db).await.map_err(Error::from)
        })
    }

    fn update<'e: 'a, E>(self, db: E) -> BoxFuture<'a, Result<Self::Model>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>,
    {
        Box::pin(async move {
            let update_fields = self.modified_fields();
            let text = format!(
                "UPDATE {} SET {} WHERE {} = {} RETURNING *",
                Self::Model::table_name(),
                update_fields.into_iter().map(|col| format!("{} = {}", col, PLACEHOLDER)).collect::<Vec<_>>().join(", "),
                Self::Model::primary_key_column(),
                self.updating.expect("Tried to call ModelBuilder::update(), but no model found to update. Call should look something like: <Model>.update_partial().update(&mut db)").id,
            );
            let mut q = sqlx::query_as::<DB, Self::Model>(&text);
            if let Some(value) = self.id {
                q = q.bind(value);
            }
            if let Some(value) = self.name {
                q = q.bind(value);
            }
            if let Some(value) = self.age {
                q = q.bind(value);
            }
            q.fetch_one(db).await.map_err(Error::from)
        })
    }
}

pub struct InsertPerson {
    name: String,
    age: u8,
}

impl<'a> HasInsertModel<'a, DB> for Person {
    type Insert = InsertPerson;
}

impl<'a> Insertable<'a, DB> for InsertPerson {
    type Model = Person;

    fn insert<'e: 'a, E>(self, db: E) -> BoxFuture<'a, Result<Self::Model>>
    where
        E: 'e + sqlx::Executor<'e, Database = DB>,
    {
        Box::pin(async move {
            let insert_fields = ["name", "age"];
            let query = format!(
                "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                Self::Model::table_name(),
                insert_fields.join(", "),
                insert_fields
                    .iter()
                    .map(|_| PLACEHOLDER)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            let mut q = sqlx::query_as::<DB, Self::Model>(&query);
            q = q.bind(self.name);
            q = q.bind(self.age);
            q.fetch_one(db).await.map_err(Error::from)
        })
    }
}
