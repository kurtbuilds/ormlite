/// This module contains everything needed from sqlx to set up a database connection or pool.
/// Using this module, it is optional to directly depend on the `sqlx` crate.
pub use sqlx::{query, query_as, Connection, Database, Executor};

#[cfg(feature = "postgres")]
pub use sqlx::postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions};
#[cfg(feature = "sqlite")]
pub use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqlitePool, SqlitePoolOptions};
