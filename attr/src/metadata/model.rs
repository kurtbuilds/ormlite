use crate::ident::Ident;
use crate::metadata::column::ColumnMetadata;
use crate::metadata::table::TableMetadata;
use crate::{ModelAttributes, SyndecodeError};
use syn::DeriveInput;

/// Metadata used for IntoArguments, TableMeta, and (subset of) Model
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub inner: TableMetadata,
    pub insert_struct: Option<String>,
    pub pkey: ColumnMetadata,
}

impl ModelMetadata {
    pub fn new(name: &str, columns: Vec<ColumnMetadata>) -> Self {
        let inner = TableMetadata::new(name, columns);
        Self {
            pkey: inner.columns.iter().find(|c| c.column_name == "id").unwrap().clone(),
            inner,
            insert_struct: None,
        }
    }
    pub fn table(&self) -> &str {
        &self.inner.table_name
    }

    pub fn struct_name(&self) -> &Ident {
        &self.inner.struct_name
    }

    pub fn builder_struct(&self) -> Ident {
        let mut s = self.inner.struct_name.0.clone();
        s.push_str("Builder");
        Ident(s)
    }

    pub fn database_columns_except_pkey(&self) -> impl Iterator<Item = &ColumnMetadata> + '_ {
        self.inner
            .columns
            .iter()
            .filter(|&c| !c.skip)
            .filter(|&c| self.pkey.column_name != c.column_name)
    }

    pub fn database_columns(&self) -> impl Iterator<Item = &ColumnMetadata> + '_ {
        self.inner.database_columns()
    }

    pub fn many_to_one_joins(&self) -> impl Iterator<Item = &ColumnMetadata> + '_ {
        self.inner.many_to_one_joins()
    }

    pub fn columns(&self) -> impl Iterator<Item = &ColumnMetadata> + '_ {
        self.inner.columns.iter()
    }

    pub fn from_derive(ast: &DeriveInput) -> Result<Self, SyndecodeError> {
        let inner = TableMetadata::from_derive(ast)?;
        let pkey = inner.pkey.clone().expect(&format!(
            "No column marked with #[ormlite(primary_key)], and no column named id, uuid, {0}_id, or {0}_uuid",
            inner.table_name,
        ));
        let pkey = inner.columns.iter().find(|&c| c.column_name == pkey).unwrap().clone();
        let mut insert_struct = None;
        for attr in ast.attrs.iter().filter(|a| a.path().is_ident("ormlite")) {
            let args: ModelAttributes = attr.parse_args().map_err(|e| SyndecodeError(e.to_string()))?;
            if let Some(value) = args.insertable {
                insert_struct = Some(value.to_string());
            }
        }
        Ok(Self {
            inner,
            insert_struct,
            pkey,
        })
    }
}

impl std::ops::Deref for ModelMetadata {
    type Target = TableMetadata;

    fn deref(&self) -> &Self::Target {
        &self.inner
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
        let meta = ModelMetadata::from_derive(&input).unwrap();
        assert_eq!(meta.pkey.column_name, "Id");
    }
}
