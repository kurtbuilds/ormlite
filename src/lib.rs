// use std::io::Write;
// use sqlx::{Connection, Encode, Error};
// use sqlx::database::HasArguments;
// use sqlx::encode::IsNull;
// use sqlx::query::Query;
// use sqlx::sqlite::SqliteRow;
//
// // // the library
// // pub enum Settable<T> {
// //     Unset,
// //     Set(T),
// //     Filter(T),
// //     FilterSet(T, T),
// // }
// //
// //
// // /** Declares that a struct can create queries. Put another way, this trait
// // is various ways to create a "builder" from the base struct.
// //  */
// // pub trait Queryable<T> {
// //     fn query(&self, query: &str) -> sqlx::query::Query<DB, Self>;
// //
// //     /**
// //     user.partial_update()
// //         .name("new name")
// //         .update(&db).await;
// //      */
// //     fn update_partial(&self) -> T;
// //
// //     fn select(&self) -> SelectQueryBuilder;
// //
// //     fn insert(&self, &mut db: dyn sqlx::Connection) -> Result<(), sqlx::error::Error>;
// //     fn update_all_columns(&self, &mut db: dyn sqlx::Connection) -> Result<(), sqlx::error::Error>;
// //     fn delete(&self, &mut db: dyn sqlx::Connection) -> Result<(), sqlx::error::Error>;
// // }
// //
// //
// // /** Declares that a struct can generate queries and execute them.
// // Put another way, this trait is defined *on* the "builder" to execute it.
// //  */
// // pub trait Executable {
// //     fn insert(&self, &mut db: dyn sqlx::Connection) -> Result<(), sqlx::error::Error>;
// //     fn update(&self, &mut db: dyn sqlx::Connection) -> Result<(), sqlx::error::Error>;
// // }
//
//
// // the user code.
// struct User {
//     id: Uuid,
//     name: String,
//     age: u32,
//     updated_at: chrono::DateTime<Utc>,
// }
//
//
// impl User {
//     fn new(name: &str) -> Self {
//         User {
//             id: Uuid::new_v4(),
//             name: name.to_string(),
//             age: 0,
//             updated_at: chrono::Utc::now(),
//         }
//     }
// }
//
//
// // the generated code.
// // needed info:
// // - the table name
// impl Queryable<PartialUser> for User {
//     fn query(&self, query: &str) -> Query<DB, Self> {
//         todo!()
//     }
//
//     fn update_partial(&self) -> PartialUser {
//         let mut partial = PartialUser::new();
//         partial.id = Settable::Filter(self.id);
//         if cfg!(feature = "updated_at_flag") {
//             partial.updated_at = Settable::Filter(Utc::now());
//         }
//         partial
//     }
//
//     fn select_builder(&self) -> SelectQueryBuilder {
//         SelectQueryBuilder {
//             table: "users".to_string(),
//             select: vec!["users.*".to_string()],
//             ..SelectQueryBuilder::default()
//         }
//     }
//
//     fn insert(&self, &mut db: dyn Connection) -> Result<(), Error> {
//         todo!()
//     }
//
//     fn update_all_columns(&self, &mut db: dyn Connection) -> Result<(), Error> {
//         todo!()
//     }
//
//     fn delete(&self, &mut db: dyn Connection) -> Result<(), Error> {
//         todo!()
//     }
// }
//
//
// struct PartialUser {
//     id: Settable<Uuid>,
//     name: Settable<String>,
//     age: Settable<u32>,
//     updated_at: Settable<chrono::DateTime<Utc>>,
// }
//
//
// impl PartialUser {
//     pub fn new() -> Self {
//         PartialUser::default()
//     }
//
//     fn id(&mut self, id: Uuid) -> &mut Self {
//         self.id = Settable::Set(id);
//         self
//     }
//
//     fn name(&mut self, name: String) -> &mut Self {
//         self.name = Settable::Set(name);
//         self
//     }
//
//     fn age(&mut self, age: u32) -> &mut Self {
//         self.age = Settable::Set(age);
//         self
//     }
//
//     fn updated_at(&mut self, updated_at: chrono::DateTime<Utc>) -> &mut Self {
//         self.updated_at = Settable::Set(updated_at);
//         self
//     }
//
//     fn build_insert(&self) {
//     }
//     fn build_update(&self) {
//         let mut r = String::new();
//         let mut n: usize = 0;
//         r += "UPDATE ";
//         r += "user ";
//         r += "\nSET ";
//         r += &*vec![
//             ("id", &self.id),
//             ("name", &self.name),
//             ("age", &self.age),
//             ("updated_at", &self.updated_at),
//         ].into_iter().filter_map(|(name, value)| {
//             if let Settable::Set(x) | Settable::FilterSet(_, x) = value {
//                 n += 1;
//                 Some(format!("{name} = ${n}"))
//             } else {
//                 None
//             }
//         }).join(", ");
//         r += "\nWHERE ";
//         //     r += &format!("id = {x}, ", x=x);
//         //     first = true
//         // }
//         // if let Settable::Set(x) | Settable::FilterSet(_, x) = &self.name {
//         //     r += &format!("name = {x}, ", x=x);
//         // }
//         }
// }
//
//
// impl Executable for PartialUser {
//     fn insert(&self, &mut db: dyn sqlx::Connection) -> Result<(), sqlx::error::Error> {
//         todo!()
//     }
//     fn update(&self, &mut db: dyn sqlx::Connection) -> Result<(), sqlx::error::Error> {
//         todo!()
//     }
// }
//
//
// impl Default for PartialUser {
//     fn default() -> Self {
//         Self {
//             id: Settable::Unset,
//             name: Settable::Unset,
//             age: Settable::Unset,
//             updated_at: Settable::Unset,
//         }
//     }
// }