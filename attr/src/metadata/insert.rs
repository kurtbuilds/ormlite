use crate::Ident;
use crate::{TableAttr, TableMeta};
use syn::DeriveInput;

/// Metadata used for IntoArguments, TableMeta, and (subset of) Model
/// This structs are constructed from the *Attribute structs in crate::attr.
#[derive(Debug, Clone)]
pub struct InsertMeta {
    pub table: TableMeta,
    pub returns: Ident,
    /// Only gets set if the table attribute was set
    pub name: Option<String>,
}

impl InsertMeta {
    pub fn from_derive(ast: &DeriveInput) -> Self {
        let attrs = TableAttr::from_attrs(&ast.attrs);
        let table = TableMeta::new(ast, &attrs);
        let mut returns = None;
        let mut name = None;
        for attr in attrs {
            if let Some(v) = attr.returns {
                returns = Some(v.value());
            }
            if let Some(v) = attr.table {
                name = Some(v.value());
            }
        }
        let returns =
            returns.expect("You must specify #[ormlite(returns = \"...\")] for structs marked with #[derive(Insert)]");
        let returns = Ident::from(returns);
        Self { table, returns, name }
    }
}

impl std::ops::Deref for InsertMeta {
    type Target = TableMeta;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

#[cfg(test)]
mod tests {
    use syn::{parse_str, ItemStruct};

    use super::*;

    #[test]
    fn test_name() {
        let s = r#"#[derive(Insert)]
        #[ormlite(returns = "User")]
        pub struct InsertUser2 {
            name: String,
            number: i32,
            ty: i32,
            org_id: i32,
            }"#;
        let s: ItemStruct = parse_str(s).unwrap();
        let s = DeriveInput::from(s);
        let meta = InsertMeta::from_derive(&s);
        assert_eq!(meta.returns, "User");
    }
}
