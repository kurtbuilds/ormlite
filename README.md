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

    /// Query building syntax is basically SQL translated into chained function calls.
    let people = Person::select()
        .where_("age > ?").bind(50)
        .fetch_all(&mut conn).await?;
    println!("{:?}", people);
}
```

> **Note**: Using this syntax, there are two possible issues. First, `id` must be set client-side instead of using the
> database's auto-increment counter, because the field is not `Option`. Second, the struct cannot track which fields are
> modified, so the update method must updates all columns. If these issues present problems for your usage, check the
> sections [Insertion Struct](#insertion-struct) or [Builder Syntax](#builder-syntax) below for alternative APIs that
> resolve these issues.

Continue reading this `README` for installation instructions, advanced examples, and more.

While the software is still 0.x, we hold a high standard for stability. Breaking changes will only be made for good
reason, with migration instructions provided. We mark affected changes with `#[deprecated]`, and provide inline
migration instructions.

`ormlite` is built on the wonderful foundation of [`sqlx`](https://github.com/launchbadge/sqlx)

# Quickstart with Migrations

`ormlite` has a CLI tool to generate migrations. To our knowledge, `ormlite` is the first, and currently only, Rust ORM
that auto-generates migrations based on changes to Rust code.

> **NOTE**: The CLI tool is under development. It works for simple cases, but it may not support all features yet. Please 
> submit an issue if you encounter any. It also only works for Postgres currently.

The CLI also has built-in functionality for database snapshots, letting you roll back locally without needing to write
(or generate) down migrations.

See the [quickstart](#cli-quickstart) section for documentation.

The `ormlite` CLI tool is 100% compatible with
[`sqlx-cli`](https://github.com/launchbadge/sqlx/blob/master/sqlx-cli/README.md#usage). The latter does not support
auto-generation or snapshots, but does support other database types, and is less bleeding edge.

# Installation

For postgres:

    [dependencies]
    ormlite = { version = "...", features = ["postgres", "runtime-tokio-rustls"]

For sqlite:

    [dependencies]
    ormlite = { version = "...", features = ["sqlite", "runtime-tokio-rustls"]

Other databases (mysql) and runtimes should work smoothly, but might not be 100% wired up yet. Please submit an issue if you encounter any.

# Project Goals

We prioritize these objectives in the project:

* **Fast**: We aim for minimal to no measurable overhead for using the ORM.
* **True to SQL**: The API leans towards using Plain Old SQL. We eschew custom query syntax so that users don't have to learn or memorize new syntax.
* **`async`-first**: We built on top of the great foundation of `sqlx`, and `ormlite` is fully `async`.
* **Explicit**: We name methods expressively to avoid confusion about what they do.

# Usage

## Insertion Struct

As noted above, on full database models, all fields must be set before insertion, which might present problems for
certain fields, notably autoincrement id fields.

You can add an attribute to generate a struct used only for insertion.

```rust
use ormlite::model::*;

#[derive(Model, Debug)]
#[ormlite(Insertable = InsertPerson)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

async fn insertion_struct_example() {  
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
use ormlite::model::*;

#[derive(Model, Debug)]
#[ormlite(table = "person")]
pub struct InsertPerson {
    pub name: String,
    pub age: i32,
}
```

## Builder Syntax

You can also use builder syntax for insertion or to update only certain fields.

```rust
use ormlite::model::*;

#[derive(Model, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

async fn builder_syntax_example() {    
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

You can use `Model::select` to build a SQL query using Rust logic.

> **Note**: Postgres's approach of using numbered dollar sign placeholders quickly breaks down when building queries. Instead, even with Postgres, use `?` for parameters,
> and `ormlite` will replace the `?` placeholders with `$` placeholders when it constructs the final query.

```rust
use ormlite::model::*;

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

## Raw Query

You can always fallback to raw queries if none of the ORM methods work for you.

```rust
use ormlite::model::*;

#[derive(Model, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

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

## Attributes

The following attributes are available:

On the struct:

- `#[ormlite(table = "<table_name>")]`: Specify the table name.
- `#[ormlite(Insertable = InsertStructName)]`: Specify the name of the struct used for insert.

See example usage below:

```rust
use ormlite::model::*;

#[derive(Model, Debug)]
#[ormlite(table = "people", Insertable = InsertPerson)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,
}
```

## Uuid and DateTime columns

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

You can use `ormlite::types::Json` for JSON or JSONB fields. For unstructured data, use `serde_json::Value` for the 
generic. Use a struct with `Deserialize + Serialize` as the generic for structured data.

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

## Logging

You can log queries using sqlx's logger: `RUST_LOG=sqlx=info`

## CLI Quickstart

First, install it:

```bash
cargo install --git https://github.com/kurtbuilds/ormlite
```

Ensure that you have DATABASE_URL set in your environment. Here, we do it with an extremely simple `.env` setup.
In general, we recommend a tool like [`just`](https://github.com/casey/just) to run commands with `.env` files. However,
this guide sources the .env to bash environment to keep it simple.

```bash
# .env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
```

```bash
# /bin/bash
export $(cat .env)
```

Now, you can setup the database:

```bash
ormlite init
```

This will create a `_sqlx_migrations` table that tracks your migrations.

Next, let's create a Rust struct with `#[derive(Model)]`, which the tool will next read to auto-generate SQL:

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

Now, we can generate the SQL:

```bash
ormlite migrate initial
```

This creates a plain SQL file in `migrations/`. Let's review it before we execute it:

```bash
# /bin/bash
cat migrations/*.sql
```

Once you're happy with the SQL, you can execute it:

```bash
ormlite up
```

And now, our database is ready to go!

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
- [ ] Autogenerate indexes for migrations
- [ ] Joins
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
