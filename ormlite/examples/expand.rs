#![allow(unused)]
#![feature(fmt_internals)]
#![feature(fmt_helpers_for_derive)]
#![feature(print_internals)]
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use ormlite::model::*;
use ormlite::Connection;

pub struct Person {
    pub id: u32,
    pub name: String,
    pub age: u8,
}
impl<'slf> ::ormlite::model::Model<'slf, ::sqlx::Sqlite> for Person {
    type ModelBuilder = PersonBuilder<'slf>;
    fn table_name() -> &'static str {
        "person"
    }
    fn fields() -> &'static [&'static str] {
        &["id", "name", "age"]
    }
    fn num_fields() -> usize {
        3usize
    }
    fn primary_key_column() -> &'static str {
        "id"
    }
    fn insert<'e, E>(self, db: E) -> ::ormlite::BoxFuture<'e, ::ormlite::Result<Self>>
    where
        E: 'e + ::ormlite::Executor<'e, Database = ::sqlx::Sqlite>,
    {
        Box::pin(async move {
            let mut q = ::ormlite::query_as::<::sqlx::Sqlite, Self>(
                "INSERT INTO \"person\" (id, name, age) VALUES (?, ?, ?) RETURNING *",
            );
            q = q.bind(self.id);
            q = q.bind(self.name);
            q = q.bind(self.age);
            q.fetch_one(db).await.map_err(::ormlite::Error::from)
        })
    }
    fn update_all_fields<'e, E>(self, db: E) -> ::ormlite::BoxFuture<'e, ::ormlite::Result<Self>>
    where
        E: 'e + ::ormlite::Executor<'e, Database = ::sqlx::Sqlite>,
    {
        Box::pin(async move {
            let mut q = ::ormlite::query_as::<_, Self>(
                "UPDATE \"person\" SET name = ?, age = ? WHERE id = ? RETURNING *",
            );
            q = q.bind(self.name);
            q = q.bind(self.age);
            q.bind(self.id)
                .fetch_one(db)
                .await
                .map_err(::ormlite::Error::from)
        })
    }
    fn delete<'e, E>(self, db: E) -> ::ormlite::BoxFuture<'e, ::ormlite::Result<()>>
    where
        E: 'e + ::ormlite::Executor<'e, Database = ::sqlx::Sqlite>,
    {
        Box::pin(async move {
            let row = ::ormlite::query("DELETE FROM \"person\" WHERE id = ?")
                .bind(self.id)
                .execute(db)
                .await
                .map_err(::ormlite::Error::from)?;
            if row.rows_affected() == 0 {
                Err(::ormlite::Error::from(::ormlite::SqlxError::RowNotFound))
            } else {
                Ok(())
            }
        })
    }
    fn get_one<'e, 'a, Arg, E>(id: Arg, db: E) -> ::ormlite::BoxFuture<'e, ::ormlite::Result<Self>>
    where
        'a: 'e,
        Arg: 'a
            + Send
            + ::ormlite::Encode<'a, ::sqlx::Sqlite>
            + ::ormlite::types::Type<::sqlx::Sqlite>,
        E: 'e + ::ormlite::Executor<'e, Database = ::sqlx::Sqlite>,
    {
        Box::pin(async move {
            ::sqlx::query_as::<::sqlx::Sqlite, Self>("SELECT * FROM \"person\" WHERE id = ?")
                .bind(id)
                .fetch_one(db)
                .await
                .map_err(::ormlite::Error::from)
        })
    }
    fn select<'args>() -> ::ormlite::query_builder::SelectQueryBuilder<'args, ::sqlx::Sqlite, Self>
    {
        ::ormlite::query_builder::SelectQueryBuilder::new(
            ::ormlite::query_builder::Placeholder::question_mark(),
        )
        .column(&{
            let res = ::std::fmt::format(::core::fmt::Arguments::new_v1(
                &["\"", "\".*"],
                &[::core::fmt::ArgumentV1::new_display(&"person")],
            ));
            res
        })
    }
    fn build() -> PersonBuilder<'static> {
        PersonBuilder::default()
    }
    fn update_partial(&'slf self) -> PersonBuilder<'slf> {
        let mut partial = PersonBuilder::default();
        partial.updating = Some(&self);
        partial
    }
    fn query(
        query: &str,
    ) -> ::ormlite::query::QueryAs<
        ::sqlx::Sqlite,
        Self,
        <::sqlx::Sqlite as ::ormlite::database::HasArguments>::Arguments,
    > {
        ::ormlite::query_as::<_, Self>(query)
    }
}
impl<'a, R: ::ormlite::Row> ::ormlite::model::FromRow<'a, R> for Person
where
    &'a str: ::ormlite::ColumnIndex<R>,
    u32: ::ormlite::decode::Decode<'a, R::Database>,
    u32: ::ormlite::types::Type<R::Database>,
    String: ::ormlite::decode::Decode<'a, R::Database>,
    String: ::ormlite::types::Type<R::Database>,
    u8: ::ormlite::decode::Decode<'a, R::Database>,
    u8: ::ormlite::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::std::result::Result<Self, ::ormlite::SqlxError> {
        let id: u32 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let age: u8 = row.try_get("age")?;
        Ok(Self { id, name, age })
    }
}
pub struct PersonBuilder<'a> {
    id: std::option::Option<u32>,
    name: std::option::Option<String>,
    age: std::option::Option<u8>,
    updating: Option<&'a Person>,
}
impl<'a> std::default::Default for PersonBuilder<'a> {
    fn default() -> Self {
        Self {
            id: None,
            name: None,
            age: None,
            updating: None,
        }
    }
}
impl<'a> PersonBuilder<'a> {
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }
    pub fn name<T: Into<String>>(mut self, name: T) -> Self {
        self.name = Some(name.into());
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
impl<'a> ::ormlite::model::ModelBuilder<'a, ::sqlx::Sqlite> for PersonBuilder<'a> {
    type Model = Person;
    fn insert<'e: 'a, E>(self, db: E) -> ::ormlite::BoxFuture<'a, ::ormlite::Result<Self::Model>>
    where
        E: 'e + ::ormlite::Executor<'e, Database = ::sqlx::Sqlite>,
    {
        Box::pin(async move {
            let mut placeholder = ::ormlite::query_builder::Placeholder::question_mark();
            let set_fields = self.modified_fields();
            let query = {
                let res = ::std::fmt::format(::core::fmt::Arguments::new_v1(
                    &["INSERT INTO \"person\" (", ") VALUES (", ") RETURNING *"],
                    &[
                        ::core::fmt::ArgumentV1::new_display(&set_fields.join(", ")),
                        ::core::fmt::ArgumentV1::new_display(
                            &set_fields
                                .iter()
                                .map(|_| placeholder.next().unwrap())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ],
                ));
                res
            };
            let mut q = ::ormlite::query_as::<::sqlx::Sqlite, Self::Model>(&query);
            if let Some(value) = self.id {
                q = q.bind(value);
            }
            if let Some(value) = self.name {
                q = q.bind(value);
            }
            if let Some(value) = self.age {
                q = q.bind(value);
            }
            q.fetch_one(db).await.map_err(::ormlite::Error::from)
        })
    }
    fn update<'e: 'a, E>(self, db: E) -> ::ormlite::BoxFuture<'a, ::ormlite::Result<Self::Model>>
    where
        E: 'e + ::ormlite::Executor<'e, Database = ::sqlx::Sqlite>,
    {
        Box::pin(async move {
            let mut placeholder = ::ormlite::query_builder::Placeholder::question_mark();
            let set_fields = self.modified_fields();
            let update_id = self
                .updating
                .expect(
                    "Tried to call ModelBuilder::update(), but the ModelBuilder \
                        has no reference to what model to update. You might have called \
                        something like: `<Model>::build().update(&mut db)`. A partial update \
                        looks something like \
                        `<model instance>.update_partial().update(&mut db)`.",
                )
                .id;
            let query = {
                let res = ::std::fmt::format(::core::fmt::Arguments::new_v1(
                    &["UPDATE \"person\" SET ", " WHERE id = ", " RETURNING *"],
                    &[
                        ::core::fmt::ArgumentV1::new_display(
                            &set_fields
                                .into_iter()
                                .map(|f| {
                                    let res = ::std::fmt::format(::core::fmt::Arguments::new_v1(
                                        &["", " = "],
                                        &[
                                            ::core::fmt::ArgumentV1::new_display(&f),
                                            ::core::fmt::ArgumentV1::new_display(
                                                &placeholder.next().unwrap(),
                                            ),
                                        ],
                                    ));
                                    res
                                })
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                        ::core::fmt::ArgumentV1::new_display(&placeholder.next().unwrap()),
                    ],
                ));
                res
            };
            let mut q = ::ormlite::query_as::<::sqlx::Sqlite, Self::Model>(&query);
            if let Some(value) = self.id {
                q = q.bind(value);
            }
            if let Some(value) = self.name {
                q = q.bind(value);
            }
            if let Some(value) = self.age {
                q = q.bind(value);
            }
            q = q.bind(update_id);
            q.fetch_one(db).await.map_err(::ormlite::Error::from)
        })
    }
}
pub struct InsertPerson {
    pub name: String,
    pub age: u8,
}
impl ::ormlite::model::Insertable<::sqlx::Sqlite> for InsertPerson {
    type Model = Person;
    fn insert<'e, E>(self, db: E) -> ::ormlite::BoxFuture<'e, ::ormlite::Result<Self::Model>>
    where
        E: 'e + ::ormlite::Executor<'e, Database = ::sqlx::Sqlite>,
    {
        Box::pin(async move {
            let mut q = ::ormlite::query_as::<::sqlx::Sqlite, Self::Model>(
                "INSERT INTO \"person\" (name,age) VALUES (?,?) RETURNING *",
            );
            q = q.bind(self.name);
            q = q.bind(self.age);
            q.fetch_one(db).await.map_err(::ormlite::Error::from)
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Person {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Person",
            "id",
            &&self.id,
            "name",
            &&self.name,
            "age",
            &&self.age,
        )
    }
}
pub static CREATE_TABLE_SQL: &str =
    "CREATE TABLE person (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = async {
        let mut conn = ormlite::SqliteConnection::connect(":memory:")
            .await
            .unwrap();
        env_logger::init();
        ormlite::query(CREATE_TABLE_SQL).execute(&mut conn).await?;
        let mut john = Person {
            id: 1,
            name: "John".to_string(),
            age: 99,
        }
        .insert(&mut conn)
        .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&john)],
            ));
        };
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(&["select\n"], &[]));
        };
        let people = Person::select()
            .where_("age > ?")
            .bind(50u32)
            .fetch_all(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["select query builder ", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&people)],
            ));
        };
        let r = sqlx::query_as::<_, Person>("select * from person where age > ?")
            .bind(50u32)
            .fetch_all(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["sqlx ", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&r)],
            ));
        };
        john.age = john.age + 1;
        john = john.update_all_fields(&mut conn).await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&john)],
            ));
        };
        john.delete(&mut conn).await?;
        Person::get_one(1u32, &mut conn)
            .await
            .expect_err("Should not exist");
        Person {
            id: 1,
            name: "Dan".to_string(),
            age: 28,
        }
        .insert(&mut conn)
        .await?;
        let dan = Person::get_one(1u32, &mut conn).await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["get_one ", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&dan)],
            ));
        };
        dan.update_partial().age(29).update(&mut conn).await?;
        InsertPerson {
            name: "Albert Einstein".to_string(),
            age: 60,
        }
        .insert(&mut conn)
        .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["build ", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&dan)],
            ));
        };
        let kurt = Person::build()
            .name("Kurt".to_string())
            .age(29)
            .insert(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["built ", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&kurt)],
            ));
        };
        let people = Person::select()
            .where_("age > ?")
            .bind(50u32)
            .fetch_all(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["select builder ", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&people)],
            ));
        };
        let people = Person::query("SELECT * FROM person WHERE age > ?")
            .bind(20u32)
            .fetch_all(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["raw query: ", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&people)],
            ));
        };
        Ok(())
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
