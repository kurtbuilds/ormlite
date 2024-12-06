use crate::metadata::column::ColumnMeta;
use crate::metadata::table::TableMeta;
use crate::Ident;
use crate::TableAttr;
use syn::DeriveInput;

/// Metadata used for IntoArguments, TableMeta, and (subset of) Model
#[derive(Debug, Clone)]
pub struct ModelMeta {
    pub table: TableMeta,
    pub insert_struct: Option<Ident>,
    pub extra_derives: Option<Vec<Ident>>,
    pub pkey: ColumnMeta,
}

impl ModelMeta {
    pub fn builder_struct(&self) -> Ident {
        Ident::from(format!("{}Builder", self.ident.as_ref()))
    }

    pub fn database_columns_except_pkey(&self) -> impl Iterator<Item = &ColumnMeta> + '_ {
        self.columns
            .iter()
            .filter(|&c| !c.skip)
            .filter(|&c| self.pkey.name != c.name)
    }

    pub fn from_derive(ast: &DeriveInput) -> Self {
        let attrs = TableAttr::from_attrs(&ast.attrs);
        let table = TableMeta::new(ast, &attrs);
        let pkey = table.pkey.as_deref().unwrap_or_else(|| {
            panic!(
                "No column marked with #[ormlite(primary_key)], and no column named id, uuid, {0}_id, or {0}_uuid",
                table.name
            )
        });
        let mut insert_struct = None;
        let mut extra_derives: Option<Vec<syn::Ident>> = None;
        for attr in attrs {
            if let Some(v) = attr.insert {
                insert_struct = Some(v.value());
            }
            if let Some(v) = attr.insertable {
                insert_struct = Some(v.to_string());
            }
            if let Some(v) = attr.extra_derives {
                if !v.is_empty() {
                    extra_derives = Some(v);
                }
            }
        }
        let pkey = table.columns.iter().find(|&c| c.name == pkey).unwrap().clone();
        fn fun_name(v: String) -> Ident {
            Ident::from(v)
        }
        let insert_struct = insert_struct.map(fun_name);
        let extra_derives = extra_derives
            .take()
            .map(|vec| vec.into_iter().map(|v| v.to_string()).map(Ident::from).collect());

        Self {
            table,
            insert_struct,
            extra_derives,
            pkey,
        }
    }

    #[doc(hidden)]
    pub fn mock(name: &str, columns: Vec<ColumnMeta>) -> Self {
        let inner = TableMeta::mock(name, columns);
        Self {
            pkey: inner.columns.iter().find(|c| c.name == "id").unwrap().clone(),
            table: inner,
            extra_derives: None,
            insert_struct: None,
        }
    }
}

impl std::ops::Deref for ModelMeta {
    type Target = TableMeta;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::ItemStruct;

    #[test]
    fn test_decode_metadata() {
        let ast = syn::parse_str::<ItemStruct>(
            r#"struct User {
            #[ormlite(column = "Id")]
            id: i32,
        }"#,
        )
        .unwrap();
        let input = DeriveInput::from(ast);
        let meta = ModelMeta::from_derive(&input);
        assert_eq!(meta.pkey.name, "Id");
    }
}
