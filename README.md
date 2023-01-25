<div id="top"></div>

<p align="center">
<a href="https://github.com/kurtbuilds/ormlite/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/kurtbuilds/ormlite.svg?style=flat-square" alt="GitHub Contributors" />
</a>
<a href="https://github.com/kurtbuilds/ormlite/stargazers">
    <img src="https://img.shields.io/github/stars/kurtbuilds/ormlite.svg?style=flat-square" alt="Stars" />
</a>
<a href="https://github.com/kurtbuilds/ormlite/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/kurtbuilds/ormlite/test.yaml?style=flat-square" alt="Build Status" />
</a>
<a href="https://crates.io/crates/ormlite">
    <img src="https://img.shields.io/crates/d/ormlite?style=flat-square" alt="Downloads" />
</a>
<a href="https://crates.io/crates/ormlite">
    <img src="https://img.shields.io/crates/v/ormlite?style=flat-square" alt="Crates.io" />
</a>

</p>

# `ormlite`

**`ormlite` is an ORM in Rust for developers that love SQL.** Let's see it in action:

```rust
use ormlite::model::*;

#[derive(Model, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// Start by making a database connection.
     let mut conn = ormlite::SqliteConnection::connect(":memory:").await.unwrap();

    /// You can insert the model directly.
    let mut john = Person {
        id: 1,
        name: "John".to_string(),
        age: 99,
    }.insert(&mut conn).await?;
    println!("{:?}", john);

    /// After modifying the object, you can update all its fields.
    john.age += 1;
    john.update_all_fields(&mut conn).await?;

    /// Query builder syntax closely follows SQL syntax, translated into chained function calls.
    let people = Person::select()
        .where_("age > ?").bind(50)
        .fetch_all(&mut conn).await?;
    println!("{:?}", people);
}
```

You might like `ormlite` because:

- It auto-generates migrations from Rust structs. To my knowledge, it is the only Rust ORM with this capability.
- The join API (in alpha) has far fewer moving pieces than any other Rust ORMs. It only relies on the table `struct`s themselves, and does not rely on relation traits (SeaORM) or modules (Diesel).
- There's little to no query builder syntax to learn. The query builder basically joins together &str fragments of raw SQL. It strikes the right level of abstraction between composability, and having near-zero learning curve for anyone who already knows SQL.

# Quickstart

### Installation

First, update your 
For postgres:

```toml
[dependencies]
# For postgres
ormlite = { version = "0.9", features = ["postgres", "runtime-tokio-rustls"]
# For sqlite
ormlite = { version = "0.9", features = ["sqlite", "runtime-tokio-rustls"]
```

Other databases are runtimes are supported, but are less tested. Please submit an issue if you encounter any.


### Environment Setup

You need `DATABASE_URL` in your environment. We recommend a tool like [`just`](https://github.com/casey/just), which
can be configured to pull in a `.env` file, but for simplicity, here we'll use your shell directly.

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
```

### Migrations

If you are querying a static database and don't need migrations, skip this section. If you want migrations, keep reading.

First, install the CLI tool. It is 100% compatible with [`sqlx-cli`](https://github.com/launchbadge/sqlx/blob/master/sqlx-cli/README.md#usage).
`sqlx-cli` does not support auto-generating migrations or snapshots (to rollback in development without writing down migrations), but it is less bleeding edge.

```bash
cargo install ormlite
```

Next, create the database and the migrations table. This creates a `_sqlx_migrations` table that tracks your migrations.

```bash
# If the database doesn't exist, create it first:
# createdb <dbname>  # for postgres
ormlite init
```

Let's see migrations in action. Create a Rust struct with `#[derive(Model)]`, which the CLI tool detects to auto-generate migrations:

```
# src/models.rs

use ormlite::model::*;

#[derive(Model, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}
```

Next, auto-generate the migration.

```bash
ormlite migrate initial
```

This creates a plain SQL file in `migrations/`. Let's review it before we execute it:

```bash
cat migrations/*.sql
```

Once you're satisfied reviewing it, you can execute it:

```bash
ormlite up
```

That's the end of setup. Let's now look at insertion.

# Insert & Update

The insert and update syntax at the top of the README is most effective for UUID primary key tables.

```rust
use ormlite::model::*;
use uuid::Uuid;

#[derive(Model, Debug)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = ormlite::SqliteConnection::connect(":memory:").await.unwrap();

    let mut event = Event {
        id: Uuid::new_v4(),
        name: "user_clicked".to_string(),
    }.insert(&mut conn).await?;
    println!("{:?}", event);
}
```

However, this syntax has two possible issues. First, `id` is not `Option`, so it must be set,
causing problems for autoincrement id fields. Second, the struct cannot track which fields are modified, so the update
method must update all columns.

### Insertion Struct

To work around the autoincrement issue, you can use an insertion struct, shown here, or a builder, shown below.

```rust
#[derive(Model, Debug)]
#[ormlite(Insertable = InsertPerson)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

async fn insertion_struct_example(conn: &mut SqliteConnection) {  
    let john: Person = InsertPerson {
        name: "John".to_string(),
        age: 99,
    }.insert(&mut conn).await?;
    println!("{:?}", john);
}
```

If the derived struct doesn't meet your needs, you can manually define a struct that only contains the fields you want,
specifying `table = "<table>"` to route the struct to the same database table.

```rust
#[derive(Model, Debug)]
#[ormlite(table = "person")]
pub struct InsertPerson {
    pub name: String,
    pub age: i32,
}
```

### Insert Builder & Update Builder

You can also use builder syntax for insertion or to update only certain fields.

```rust
#[derive(Model, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

async fn builder_syntax_example() {    
    // builder syntax for insert
    let john = Person::builder()
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

# Select Query

You can use `Model::select` to build a SQL query using Rust logic.

> **Note**: Postgres's approach of using numbered dollar sign placeholders quickly breaks down when building queries. Instead, even with Postgres, use `?` for parameters,
> and `ormlite` will replace the `?` placeholders with `$` placeholders when it constructs the final query.

```rust
#[derive(Model, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

async fn query_builder_example() {
    let people = Person::select()
        .where_("age > ?")
        .bind(50i32)
        .fetch_all(&mut conn)
        .await?;
    println!("All people over 50: {:?}", people); 
}
```

### Raw Query

You can fall back to raw queries if the ORM methods don't work for you. You can include handwritten strings, or if
you want a query builder that you have full control over, you can use [`sqlmo`](https://github.com/kurtbuilds/sqlmo),
the engine which powers `ormlite`'s query builder (& migration auto-generation).

```rust
async fn model_query_example() {
    // Query using the Model to still deserialize results into the struct
    let _person = Person::query("SELECT * FROM person WHERE id = ?")
        .bind(1)
        .fetch_one(&mut conn)
        .await?;
}

async fn raw_query_example() {
    // You can also use the raw query API, which will return tuples to decode as you like
    let _used_ids: Vec<i32> = ormlite::query("SELECT id FROM person")
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row: (i32, )| row.0)
        .collect();
}
```

# Table Customization

Attributes are defined in [these structs](https://github.com/kurtbuilds/ormlite/blob/master/attr/src/attr.rs).

This example shows them in action:

```rust
#[derive(Model, Debug)]
#[ormlite(table = "people", Insertable = InsertPerson)]
pub struct Person {
    #[ormlite(primary_key)]
    pub id: i32,
    pub name: String,
    #[ormlite(column = "name_of_column_in_db")]
    pub age: i32,
}
```

## Joins

Join support is alpha stage. Right now, we only support many-to-one relations (e.g. Person belongs to Organization). 
Support for many-to-many and one-to-many is planned. If you use this functionality, we appreciate reports on any 
bugs reports you encounter.

```rust
#[derive(Model, Debug)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub age: i32,
    
    pub organization_id: Uuid,
    
    #[ormlite(many_to_one_key = organization_id)]
    pub organization: Join<Organization>,
}

#[derive(Model, Debug)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note we don't need to insert it.
    let org = Organization {
        id: Uuid::new_v4(),
        name: "Acme".to_string(),
    };
    
    let user = Person {
        id: Uuid::new_v4(),
        name: "John".to_string(),
        age: 99,
        // Note we don't need to set this field. It'll be overwritten by the Join.
        organization_id: Uuid::default(),
        organization: Join::new(org),
    };
    
    let mut conn = ormlite::SqliteConnection::connect(":memory:").await.unwrap();
    let user = user.insert(&mut conn).await?;
    assert!(user.organization.loaded());
    println!("{:?}", user);
    
    // You can choose whether you want to load the relation or not. The value will be Join::NotQueried if you don't 
    // opt-in to loading it.
    let users = Person::select()
        .join(Person::organization())
        .fetch_all(&mut conn)
        .await?;
    for user in users {
        assert!(user.organization.loaded());
        println!("{:?}", user);
    }
}
```

# Features & Data Types

## Uuid, Chrono, & Time

If you want Uuid or DateTime, combined with serde, you need to depend directly on `uuid`, `time` or `chrono`, 
and add the `serde` feature to each of them.

```
# Cargo.toml
[dependencies]
uuid = { version = "...", features = ["serde"] } 
chrono = { version = "...", features = ["serde"] }
time = { version = "...", features = ["serde"] }
```

```rust
use ormlite::model::*;
use serde::{Serialize, Deserialize};
use ormlite::types::Uuid;
use ormlite::types::chrono::{DateTime, Utc};

#[derive(Model, Debug, Serialize, Deserialize)]
pub struct Person {
    pub uuid: Uuid,
    pub created_at: DateTime<Utc>,
    pub name: String,
}
```

## Json/Jsonb Columns

You can use `ormlite::types::Json` for JSON or JSONB fields. For unstructured data, use `serde_json::Value` as the inner
type. Use a struct with `Deserialize + Serialize` as the generic for structured data.

```rust
use ormlite::model::*;
use ormlite::types::Json;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct JobData {
    pub name: String,
}

#[derive(Model, Serialize, Deserialize)]
pub struct Job {
    pub id: i32,
    pub structured_data: Json<JobData>,
    pub unstructured_data: Json<Value>,
}
```

# Logging

You can log queries using sqlx's logger: `RUST_LOG=sqlx=info`

# Roadmap
- [x] Insert, update, delete directly on model instances
- [x] Builder for partial update and insertions
- [x] User can create insert models that ignore default values
- [x] Select query builder
- [x] Build the derive macro
- [x] Get() function for fetching a single entity.
- [x] Ability to specify the name of a table and name of primary column
- [x] Automatically generate insert models
- [x] Automatically generate migrations
- [x] Eliminate need for FromRow macro
- [x] Many to one joins
- [ ] Autogenerate indexes for migrations
- [ ] Many to many joins
- [ ] One to many joins
- [ ] Make sure features are wired up correctly to support mysql and different runtimes & SSL libraries.
- [ ] Macro option to auto adjust columns like updated_at
- [ ] Upsert functionality
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
