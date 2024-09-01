use crate::{TableAttr, TableMeta};
use proc_macro2::Ident;
use syn::DeriveInput;

/// Metadata used for IntoArguments, TableMeta, and (subset of) Model
/// This structs are constructed from the *Attribute structs in crate::attr.
#[derive(Debug, Clone)]
pub struct InsertMeta {
    pub table: TableMeta,
    pub returns: Ident,
}

impl InsertMeta {
    pub fn from_derive(ast: &DeriveInput) -> Self {
        let attrs = TableAttr::from_attrs(&ast.attrs);
        let table = TableMeta::new(ast, &attrs);
        let mut returns = None;
        for attr in attrs {
            if let Some(v) = attr.insert {
                returns = Some(v);
            }
        }
        let returns = returns.expect("You must specify #[ormlite(returns = \"...\")] for structs marked with #[derive(Insert)]");
        let returns = Ident::new(&returns.value(), proc_macro2::Span::call_site());
        Self { table, returns }
    }
}