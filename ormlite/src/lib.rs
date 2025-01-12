#![cfg_attr(docsrs, feature(doc_cfg))]
pub use model::{FromRow, Insert, IntoArguments, Model, TableMeta};
pub use ormlite_core::BoxFuture;
pub use ormlite_core::{Error, Result};
pub use ormlite_macro::Enum;
pub use sqlx::{Column, ColumnIndex, Database, Decode, Row};
pub use tokio_stream::StreamExt;

pub use sqlx::pool::PoolOptions;
pub use sqlx::{
    query, query_as, query_as_with, query_with, Acquire, Arguments, ConnectOptions, Connection, Encode, Executor, Pool,
};

pub mod model;

pub mod query_builder {
    pub use ormlite_core::insert::OnConflict;
    pub use ormlite_core::query_builder::{Placeholder, QueryBuilderArgs, SelectQueryBuilder};
}

pub mod types {
    pub use ormlite_macro::ManualType;
    pub use sqlx::types::*;
}

pub mod decode {
    pub use sqlx::decode::*;
}

pub use sqlx::Error as SqlxError;

pub mod database {
    pub use sqlx::database::*;
}

/// We need objects available for proc-macros that aren't meant to be available to end users. This module does that.
#[doc(hidden)]
pub mod __private {
    pub use ormlite_core::insert::Insertion;
    pub use ormlite_core::join::{JoinDescription, SemanticJoinType};
    pub use sqlmo::query::Values;
    pub use sqlmo::Insert;
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
