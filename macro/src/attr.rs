use derive_builder::Builder;
use structmeta::StructMeta;
use syn::{Ident, LitStr, Type};

/// Available attributes on a struct
#[derive(StructMeta, Debug)]
pub struct ModelAttributes {
    pub table: Option<LitStr>,
    pub insert: Option<Ident>,
}

/// Available attributes on a column (struct field)
#[derive(StructMeta, Debug)]
pub struct ColumnAttributes {
    pub primary_key: bool,
    pub default: bool,
}