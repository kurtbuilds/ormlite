pub use ormlite_core::model::*;
pub use ormlite_core::SelectQueryBuilder;
pub use sqlx::sqlx_macros::FromRow;
pub use ormlite_macro::Model;

#[deprecated(note = "This trait is part of ormlite::Model now.")]
pub trait HasQueryBuilder {}