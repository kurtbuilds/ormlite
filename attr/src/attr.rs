use structmeta::StructMeta;
use syn::{Ident, LitStr, Path};

/// Available attributes on a struct
#[derive(StructMeta, Debug)]
pub struct ModelAttributes {
    pub table: Option<LitStr>,
    pub Insertable: Option<Ident>,
}

/// Available attributes on a column (struct field)
#[derive(StructMeta, Debug)]
pub struct ColumnAttributes {
    pub primary_key: bool,
    pub default: bool,
    pub many_to_one_key: Option<Path>,
    pub many_to_many_table_name: Option<Path>,
    pub one_to_many_foreign_key: Option<Path>,
}