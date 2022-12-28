pub use ormlite_core::BoxFuture;
pub use ormlite_core::{Error, Result};
pub use ormlite_macro::Model;
pub use sqlx::sqlx_macros::FromRow;

pub use sqlx::{query, query_as, Connection, Executor, Pool};

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub use sqlx::postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions};
#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqlitePool, SqlitePoolOptions};

pub mod model;

pub mod query_builder {
    pub use ormlite_core::query_builder::{SelectQueryBuilder, Placeholder};
}

pub mod types {
    pub use sqlx::types::*;
}

// #[deprecated(note = "Most objects in ormlite::export:: are directly in ormlite::* now.")]
pub mod export {
    #[cfg(feature = "postgres")]
    pub use sqlx::postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions};
}

pub struct Foo;