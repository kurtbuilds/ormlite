pub use ormlite_core::BoxFuture;
pub use ormlite_core::{Error, Result};
pub use model::{Model, FromRow};
pub use ::sqlx::{Row, ColumnIndex, Decode};

pub use ::sqlx::{query, query_as, Connection, Executor, Pool, Acquire, ConnectOptions, Encode, Arguments, query_with};
pub use ::sqlx::pool::PoolOptions;

pub mod model;

#[deprecated(note = "Most objects in ormlite::export:: are directly in ormlite::* now.")]
#[doc(hidden)]
pub mod query_builder {
    pub use ormlite_core::query_builder::{SelectQueryBuilder, Placeholder};
}

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
    #[doc(hidden)]
    pub use sqlx::postgres::{PgConnectOptions, PgConnection, PgPool, PgPoolOptions};
}

#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub mod postgres {
    pub use sqlx::postgres::*;
}

#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub mod sqlite {
    pub use sqlx::sqlite::*;
}
