# `ormlite`

**`ormlite` is an ORM in Rust for developers that love SQL.**

```rust
use ormlite::model::*;

#[derive(Model, FromRow, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// Start by making a database connection.
    let mut conn = ormlite::SqliteConnection::connect_with(&sqlx::sqlite::SqliteConnectOptions::from_str("sqlite://:memory:").unwrap()).await?;

    /// You can insert the model directly.
    let mut john = Person {
        id: 1,
        name: "John".to_string(),
        age: 99,
    }.insert(&mut conn).await;
    println!("{:?}", john);

    /// After modifying the object, you can update all its fields.
    john.age += 1;
    john.update_all_fields(&mut conn).await?;

    let people = Person::select()
        .filter("age > ?").bind(50)
        .fetch_all(&mut conn).await?;
    println!("{:?}", people);
}
```

Note that the model id must be set client-side because it is not `Option`, and it cannot track which fields are
modified, so the update method updates all columns. If these present problems for your usage, check the
sections [Insertion Struct](#insertion-struct) or [Builder Syntax](#builder-syntax) below for alternative APIs that
resolve these issues.

# Installation

For postgres:

    [dependencies]
    ormlite = { version = "0.1.0", features = ["postgres", "runtime-tokio-rustls"]

For sqlite:

    [dependencies]
    ormlite = { version = "0.1.0", features = ["sqlite", "runtime-tokio-rustls"]

Other databases (mysql) and runtimes should work smoothly, but might not be 100% wired up yet. Please submit an issue if you encounter issues.


# Project Goals

We prioritize these objectives in the project:

* **Fast**: We aim for minimal to no measurable overhead for using the ORM.
* **True to SQL**: The API leans towards using Plain Old SQL. We eschew custom query syntax so that users don't have to learn or memorize new syntax.
* **`async`-first**: We built on top of the great foundation of `sqlx`, and `ormlite` is fully `async`.
* **Explicit**: We name methods expressively to avoid confusion about what they do.

# Usage

## Insertion Struct

As noted above, all fields must be set before insertion, which is a problem for certain fields, particularly
autoincrement id fields.

You can derive an struct that only contains some fields, to be used for insertion.

```rust
use ormlite::model::*;

#[derive(Model, FromRow, Debug)]
#[ormlite(insert = InsertPerson)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = ormlite::SqliteConnection::connect_with(&sqlx::sqlite::SqliteConnectOptions::from_str("sqlite://:memory:").unwrap()).await?;
    
    let john: Person = InsertPerson {
        name: "John".to_string(),
        age: 99,
    }.insert(&mut conn).await?;
    println!("{:?}", john);
}
```

If the derived struct doesn't meet your needs for some reason, you can of course manually define a struct that only contains the fields you want.

```rust
use ormlite::model::*;

#[derive(Model, FromRow, Debug)]
#[ormlite(table = "person")]
pub struct InsertPerson {
    pub name: String,
    pub age: i32,
}
```

## Builder Syntax

You can use builder syntax for insertion or to update only certain fields.

```rust
use ormlite::model::*;

#[derive(Model, FromRow, Debug)]
#[ormlite()]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = ormlite::SqliteConnection::connect_with(&sqlx::sqlite::SqliteConnectOptions::from_str("sqlite://:memory:").unwrap()).await?;
    
    // builder syntax for insert
    let john = Person::build()
        .name("John".to_string())
        .age(99)
        .insert(&mut conn).await?;
    println!("{:?}", john);

    // builder syntax for update
    let john = john.update_partial()
        .age(100)
        .update(&mut conn).await?;
    println!("{:?}", john);
}
```

## Query Builder

You can use `Model::select` to build a query, freely interspersing your SQL with your own Rust logic, conditionals, loops, and so on.

Postgres's dollar substitution quickly breaks down when building queries. Instead, even with Postgres, use `?` for parameters,
and `ormlite` will replace the `?` placeholders with `$` parameter placeholders when it constructs the final query.

```rust
use ormlite::model::*;

#[derive(Model, FromRow, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

async fn query_builder_example() {
    let people = Person::select()
        .filter("age > ?")
        .bind(50i32)
        .fetch_all(&mut conn)
        .await?;
    println!("all people over 50: {:?}", people); 
}
```

## Raw Query

You can always fallback to raw queries if none of the ORM methods work for you.

```rust
use ormlite::model::*;

#[derive(Model, FromRow, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

async fn model_query_example() {
    let _person = Person::query("SELECT * FROM person WHERE id = ?")
        .bind(1)
        .fetch_one(&mut conn)
        .await?;
}

async fn raw_query_example() {
    let _used_ids: Vec<i32> = ormlite::query("SELECT id FROM person")
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row: (i32, )| row.0)
        .collect();
}
```

## Attributes

The following attributes are available:

On the struct:

- `#[ormlite(table = "table_name")]`: Specify the table name.

See example usage below:

```rust
use ormlite::model::*;

#[derive(Model, FromRow, Debug)]
#[ormlite(table = "people")]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}
```

## Uuid and DateTime columns

If you want Uuid or DateTime, combined with serde, you need to depend directly on `uuid` (specifically version 0.8)
or `chrono` crates respectively, and add the `serde` feature.

```
# Cargo.toml
[dependencies]
# Note that this version of uuid is old, as sqlx still depends on 0.8.
# When sqlx updates, this can be updated too.
uuid = { version = "0.8", features = ["serde"] } 
chrono = { version = "0.4.19", features = ["serde"] }
```

```rust
use ormlite::model::*;
use serde::{Serialize, Deserialize};
use sqlx::types::Uuid;
use sqlx::types::chrono::{DateTime, Utc};


#[derive(Model, FromRow, Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub name: String,
}
```

## Json/Jsonb Columns

You can use `sqlx::types::Json` for JSON or JSONB fields. The parameterized type can be unstructured, using the `serde_json::Value` type, or a
specific serializable struct.

```rust
use ormlite::model::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct JobData {
    pub name: String,
    
}

#[derive(Model, FromRow, Serialize, Deserialize)]
pub struct Job {
    pub id: i32,
    pub structured_data: Json<JobData>,
    pub unstructured_data: Json<Value>,
}
```

## Logging

You can log queries using sqlx's logger: `RUST_LOG=sqlx=info`

## Migrations

`ormlite` builds upon [`sqlx`](https://github.com/launchbadge/sqlx). Use the [`sqlx-cli`](https://github.com/launchbadge/sqlx/blob/master/sqlx-cli/README.md#usage) tool for migrations.

Currently, we don't support auto-generating migrations, like you might be used to if you come from Python's alembic, Node's Typeorm, or other ORM libraries with this functionality.

# Roadmap
- [x] Insert, update, delete directly on model instances
- [x] Builder for partial update and insertions
- [x] User can create insert models that ignore default values
- [x] Select query builder
- [x] Build the derive macro
- [x] Get() function for fetching a single entity.
- [x] Ability to specify the name of a table and name of primary column
- [x] Automatically generate insert models
- [ ] Automatically generate migrations
- [ ] Make sure features are wired up correctly to support mysql and different runtimes & SSL libraries.
- [ ] Macro option to auto adjust columns like updated_at
- [ ] Upsert functionality
- [ ] Joins
- [ ] Bulk insertions
- [ ] Query builder for bulk update
- [ ] Handle on conflict clauses for bulk update
- [ ] Benchmarks against raw sql, sqlx, ormx, seaorm, sqlite3-sys, pg, diesel
- [ ] Macro option to delete with deleted_at rather than `DELETE`
- [ ] Support for patch records, i.e. update with static fields.
- [ ] Consider a blocking interface, perhaps for sqlite/Rusqlite only.

# Contributing

Open source thrives on contributions, and `ormlite` is a community project. We welcome you to file bugs, feature
requests, requests for better docs, pull requests, and more!