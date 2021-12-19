use std::borrow::Cow;
use std::io::Write;
use sqlx::{Connection, Encode, Row, Sqlite, Type};
use sqlx::encode::IsNull;
use sqlx::sqlite::{SqliteArgumentValue, SqliteTypeInfo};
use crate::select::SelectQueryBuilder;

pub mod select;


#[derive(Debug, Clone)]
struct Person {
    id: i32,
    name: String,
    // age: u32,
    // updated_at: chrono::DateTime<Utc>,
}

impl Person {
    fn new(name: &str) -> Self {
        Person {
            id: 100,
            name: name.to_string(),
            // age: 0,
            // updated_at: chrono::Utc::now(),
        }
    }

    fn select_builder() -> SelectQueryBuilder<Self> {
        SelectQueryBuilder {
            table: "users".to_string(),
            select: vec!["users.*".to_string()],
            ..SelectQueryBuilder::default()
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
    let row: (i64, ) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&mut conn).await?;
    println!("{}", row.0);
    sqlx::query("CREATE TABLE person (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL
                  )").execute(&mut conn).await?;
    sqlx::query("INSERT INTO person (name) VALUES ('Alice')")
        .execute(&mut conn).await?;
    sqlx::query("INSERT INTO person (name) VALUES ($1)")
        .bind("Bob").execute(&mut conn).await?;

    let results = Person::select_builder()
        .fetch_all(&mut conn).await?;
    println!("{:?}", results);

    let rows = sqlx::query("SELECT * from person")
        .fetch_all(&mut conn).await?;
    let names = rows.iter().map(|row| {
        row.get(1)
    }).collect::<Vec<String>>();
    println!("{:?}", names);
    Ok(())
}
