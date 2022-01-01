use derive_builder::Builder;
use structmeta::StructMeta;
use syn::{Ident, LitStr, Type};

#[derive(StructMeta, Debug)]
pub struct ModelAttributes {
    pub table: Option<LitStr>,
    pub insert: Option<Ident>,
}

#[derive(StructMeta, Debug)]
pub struct ColumnAttributes {
    pub primary_key: bool,
    pub default: bool,
}

#[derive(Builder, Debug)]
#[builder(field(public))]
pub struct TableMeta {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Vec<ColumnMeta>,
    pub insert_struct: Option<String>,
}

#[derive(Clone, Debug, Builder)]
#[builder(field(public))]
pub struct ColumnMeta {
    pub column_name: String,
    pub column_type: Type,
    pub marked_primary_key: bool,
    pub has_database_default: bool,
}
