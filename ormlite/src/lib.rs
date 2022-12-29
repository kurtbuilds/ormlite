pub use ormlite_core::BoxFuture;
pub use ormlite_core::{Error, Result};
pub use ormlite_macro::Model;
pub use ::sqlx::{Row, ColumnIndex, Decode};

pub use ::sqlx::{query, query_as, Connection, Executor, Pool, Acquire, ConnectOptions, Encode};
pub use ::sqlx::pool::PoolOptions;

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub use ::sqlx::postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions, Postgres};
#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub use ::sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqlitePool, SqlitePoolOptions};

pub mod model;

#[deprecated(note = "Most objects in ormlite::export:: are directly in ormlite::* now.")]
pub mod query_builder {
    pub use ormlite_core::query_builder::{SelectQueryBuilder, Placeholder};
}

// #[deprecated(note = "You no longer need #[derive(FromRow)], only #[derive(ormlite::Model)].")]
// pub use ::sqlx::sqlx_macros::FromRow;

pub mod types {
    pub use sqlx::types::*;
}

pub mod decode {
    pub use sqlx::decode::*;
}

pub use sqlx::{Error as SqlxError};

pub mod database {
    pub use sqlx::database::*;
}

#[deprecated(note = "Most objects in ormlite::export:: are directly in ormlite::* now.")]
pub mod export {
    #[cfg(feature = "postgres")]
    pub use sqlx::postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions};
}
