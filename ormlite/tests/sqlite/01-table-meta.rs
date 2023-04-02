#![allow(unused)]
use uuid::Uuid;
use ormlite::TableMeta;
use serde::Serialize;

#[derive(TableMeta)]
pub struct Person {
    id: Uuid,
    name: String,
    age: u8,
}

#[derive(TableMeta, Serialize)]
pub struct Person2 {
    id: Uuid,
    #[ormlite(column = "old")]
    age: u8,
}

fn main() {
    assert_eq!(Person::table_name(), "person");
    assert_eq!(Person::table_columns(), &["id", "name", "age"]);
    assert_eq!(Person::primary_key(), Some("id"));

    assert_eq!(Person2::table_columns(), &["id", "old"]);
}
