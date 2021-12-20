#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use ormlite::Model;
use ormlite_core::model::{BuildsPartialModel, BuildsQueryBuilder, Model, PartialModel};
use sqlx::Connection;
use std::str::FromStr;
pub struct Person {
    pub id: u32,
    pub name: String,
    pub age: u8,
}
impl ::ormlite::model::Model<::sqlx::Sqlite> for Person {
    fn insert(
        self,
        db: &mut <::sqlx::Sqlite as ::sqlx::Database>::Connection,
    ) -> ::ormlite_core::BoxFuture<::ormlite::Result<Self>> {
        Box::pin(async move {
            let mut q = sqlx::query_as::<::sqlx::Sqlite, Self>(
                "INSERT INTO person (id, name, age) VALUES (?, ?, ?) RETURNING *",
            );
            q = q.bind(self.id);
            q = q.bind(self.name);
            q = q.bind(self.age);
            q.fetch_one(db).await.map_err(::ormlite::Error::from)
        })
    }
    fn update_all_fields(
        self,
        db: &mut <::sqlx::Sqlite as ::sqlx::Database>::Connection,
    ) -> ::ormlite_core::BoxFuture<::ormlite::Result<Self>> {
        Box::pin(async move {
            let mut q = sqlx::query_as::<_, Self>(
                "UPDATE person SET name = ?, age = ? WHERE id = ? RETURNING *",
            );
            q = q.bind(self.name);
            q = q.bind(self.age);
            q.bind(self.id)
                .fetch_one(db)
                .await
                .map_err(::ormlite::Error::from)
        })
    }
    fn delete(
        self,
        db: &mut <::sqlx::Sqlite as ::sqlx::Database>::Connection,
    ) -> ::ormlite_core::BoxFuture<::ormlite::Result<()>> {
        Box::pin(async move {
            let row = ::sqlx::query("DELETE FROM person WHERE id = ?")
                .bind(self.id)
                .execute(db)
                .await
                .map_err(::ormlite::Error::from)?;
            if row.rows_affected() == 0 {
                Err(::ormlite::Error::from(::sqlx::Error::RowNotFound))
            } else {
                Ok(())
            }
        })
    }
    fn get_one<'db, 'arg: 'db, T>(
        id: T,
        db: &'db mut <::sqlx::Sqlite as ::sqlx::Database>::Connection,
    ) -> ::ormlite_core::BoxFuture<'db, ::ormlite::Result<Self>>
    where
        T: 'arg + Send + for<'r> ::sqlx::Encode<'r, ::sqlx::Sqlite> + ::sqlx::Type<::sqlx::Sqlite>,
    {
        Box::pin(async move {
            ::sqlx::query_as::<::sqlx::Sqlite, Self>("SELECT * FROM person WHERE id = ?")
                .bind(id)
                .execute(db)
                .await
                .map_err(::ormlite::Error::from)
        })
    }
    fn query(
        query: &str,
    ) -> ::sqlx::query::QueryAs<
        ::sqlx::Sqlite,
        Self,
        <::sqlx::Sqlite as ::sqlx::database::HasArguments>::Arguments,
    > {
        ::sqlx::query_as::<_, Self>(query)
    }
}
impl ::ormlite::model::TableMeta for Person {
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
}
impl<'a> ormlite::model::BuildsPartialModel<'a, PartialPerson<'a>> for Person {
    fn build() -> PartialPerson<'a> {
        PartialPerson::default()
    }
    fn update_partial(&'a self) -> PartialPerson<'a> {
        let mut partial = PartialPerson::default();
        partial.updating = Some(&self);
        partial
    }
}
impl
    ::ormlite::model::BuildsQueryBuilder<
        ::sqlx::Sqlite,
        ::std::boxed::Box<dyn Iterator<Item = String>>,
    > for Person
{
    fn select<'args>(
    ) -> ::ormlite::SelectQueryBuilder<'args, ::sqlx::Sqlite, Self, Box<dyn Iterator<Item = String>>>
    {
        ::ormlite::SelectQueryBuilder::default().column(&{
            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                &["", ".*"],
                &match (&"person",) {
                    _args => [::core::fmt::ArgumentV1::new(
                        _args.0,
                        ::core::fmt::Display::fmt,
                    )],
                },
            ));
            res
        })
    }
}
pub struct PartialPerson<'a> {
    id: std::option::Option<u32>,
    name: std::option::Option<String>,
    age: std::option::Option<u8>,
    updating: Option<&'a Person>,
}
impl<'a> std::default::Default for PartialPerson<'a> {
    fn default() -> Self {
        Self {
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
impl<'a> ::ormlite::model::PartialModel<'a, ::sqlx::Sqlite> for PartialPerson<'a> {
    type Model = Person;
    fn insert<'db: 'a>(
        self,
        db: &'db mut <::sqlx::Sqlite as ::sqlx::Database>::Connection,
    ) -> ::ormlite_core::BoxFuture<'a, ::ormlite::Result<Self::Model>> {
        Box::pin(async move {
            let mut placeholder = std::iter::repeat("?".to_string());
            let set_fields = self.modified_fields();
            let query = {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["INSERT INTO person (", ") VALUES (", ")"],
                    &match (
                        &set_fields.join(", "),
                        &set_fields
                            .iter()
                            .map(|_| placeholder.next().unwrap())
                            .collect::<Vec<_>>()
                            .join(", "),
                    ) {
                        _args => [
                            ::core::fmt::ArgumentV1::new(_args.0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(_args.1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            };
            let mut q = ::sqlx::query_as::<::sqlx::Sqlite, Self::Model>(&query);
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
    fn update<'db: 'a>(
        self,
        db: &'db mut <::sqlx::Sqlite as ::sqlx::Database>::Connection,
    ) -> ::ormlite_core::BoxFuture<'a, ::ormlite::Result<Self::Model>> {
        Box::pin(async move {
            let mut placeholder = std::iter::repeat("?".to_string());
            let set_fields = self.modified_fields();
            let query = {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["UPDATE person SET ", " WHERE id = "],
                    &match (
                        &set_fields
                            .into_iter()
                            .map(|f| {
                                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                    &["", " = "],
                                    &match (&f, &placeholder.next().unwrap()) {
                                        _args => [
                                            ::core::fmt::ArgumentV1::new(
                                                _args.0,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                _args.1,
                                                ::core::fmt::Display::fmt,
                                            ),
                                        ],
                                    },
                                ));
                                res
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                        &self
                            .updating
                            .expect(
                                "Tried to call PartialModel::update(), but the PartialModel \
                            has no reference to what model to update. You might have called \
                            something like: `<Model>::build().update(&mut db)`. A partial update \
                            looks something like \
                            `<model instance>.update_partial().update(&mut db)`.",
                            )
                            .id,
                    ) {
                        _args => [
                            ::core::fmt::ArgumentV1::new(_args.0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(_args.1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            };
            let mut q = ::sqlx::query_as::<::sqlx::Sqlite, Self::Model>(&query);
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
}
#[automatically_derived]
impl<'a, R: ::sqlx::Row> ::sqlx::FromRow<'a, R> for Person
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    u32: ::sqlx::decode::Decode<'a, R::Database>,
    u32: ::sqlx::types::Type<R::Database>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    u8: ::sqlx::decode::Decode<'a, R::Database>,
    u8: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        let id: u32 = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let age: u8 = row.try_get("age")?;
        ::std::result::Result::Ok(Person { id, name, age })
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for Person {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            Person {
                id: ref __self_0_0,
                name: ref __self_0_1,
                age: ref __self_0_2,
            } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "Person");
                let _ = ::core::fmt::DebugStruct::field(debug_trait_builder, "id", &&(*__self_0_0));
                let _ =
                    ::core::fmt::DebugStruct::field(debug_trait_builder, "name", &&(*__self_0_1));
                let _ =
                    ::core::fmt::DebugStruct::field(debug_trait_builder, "age", &&(*__self_0_2));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
fn main() -> core::result::Result<(), Box<dyn std::error::Error>> {
    let body = async {
        let mut conn = sqlx::SqliteConnection::connect_with(
            &sqlx::sqlite::SqliteConnectOptions::from_str("sqlite://:memory:").unwrap(),
        )
        .await?;
        env_logger::init();
        sqlx::query(ormlite::handwritten::CREATE_TABLE_SQL)
            .execute(&mut conn)
            .await?;
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
                &match (&john,) {
                    _args => [::core::fmt::ArgumentV1::new(
                        _args.0,
                        ::core::fmt::Debug::fmt,
                    )],
                },
            ));
        };
        let people = Person::select()
            .filter("age > ?")
            .bind(50u32)
            .fetch_all(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["select query builder ", "\n"],
                &match (&people,) {
                    _args => [::core::fmt::ArgumentV1::new(
                        _args.0,
                        ::core::fmt::Debug::fmt,
                    )],
                },
            ));
        };
        let r = sqlx::query_as::<_, Person>("select * from person where age > ?")
            .fetch_all(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["sqlx ", "\n"],
                &match (&r,) {
                    _args => [::core::fmt::ArgumentV1::new(
                        _args.0,
                        ::core::fmt::Debug::fmt,
                    )],
                },
            ));
        };
        john.age = john.age + 1;
        john = john.update_all_fields(&mut conn).await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["", "\n"],
                &match (&john,) {
                    _args => [::core::fmt::ArgumentV1::new(
                        _args.0,
                        ::core::fmt::Debug::fmt,
                    )],
                },
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
                &["", "\n"],
                &match (&dan,) {
                    _args => [::core::fmt::ArgumentV1::new(
                        _args.0,
                        ::core::fmt::Debug::fmt,
                    )],
                },
            ));
        };
        dan.update_partial().age(29).update(&mut conn).await?;
        Person::build()
            .name("Kurt".to_string())
            .age(29)
            .insert(&mut conn)
            .await?;
        let people = Person::select()
            .filter("age > ?")
            .bind(50u32)
            .fetch_all(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["select builder ", "\n"],
                &match (&people,) {
                    _args => [::core::fmt::ArgumentV1::new(
                        _args.0,
                        ::core::fmt::Debug::fmt,
                    )],
                },
            ));
        };
        let people = Person::query("SELECT * FROM person WHERE age > ?")
            .bind(20u32)
            .fetch_all(&mut conn)
            .await?;
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["raw query: ", "\n"],
                &match (&people,) {
                    _args => [::core::fmt::ArgumentV1::new(
                        _args.0,
                        ::core::fmt::Debug::fmt,
                    )],
                },
            ));
        };
        Ok(())
    };
    #[allow(clippy::expect_used)]
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(body)
}
