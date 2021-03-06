use ormlite::{FromRow};
use ormlite::model::HasModelBuilder;
use ormlite::model::ModelBuilder;
use ormlite::model::Model;
use ormlite::Connection;
use uuid::Uuid;

#[derive(Model, FromRow)]
pub struct Person {
    id: Uuid,
    name: String,
    age: u8,
}

pub static CREATE_TABLE_SQL: &str =
    "CREATE TABLE person (id text PRIMARY KEY, name TEXT, age INTEGER)";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut db = ormlite::SqliteConnection::connect(":memory:").await.unwrap();
    ormlite::query(CREATE_TABLE_SQL)
        .execute(&mut db)
        .await?;

    let p = Person {
        id: Uuid::new_v4(),
        name: "John".to_string(),
        age: 99,
    }.insert(&mut db).await?;

    let p = p.update_partial()
        .age(100)
        .update(&mut db)
        .await?;

    assert_eq!(p.age, 100);
    Ok(())
}
