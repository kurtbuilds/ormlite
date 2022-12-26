pub mod common;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "mysql")]
compile_error!("mysql is currently not supported");
#[cfg(feature = "postgres")]
pub type DB = postgres::PostgresBackend;
#[cfg(feature = "sqlite")]
pub type DB = sqlite::SqliteBackend;
