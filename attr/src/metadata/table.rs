use crate::ident::Ident;
use crate::metadata::column::ColumnMetadata;
use crate::{DeriveInputExt, ModelAttributes, SyndecodeError};
use convert_case::{Case, Casing};
use syn::DeriveInput;

/// Metadata used for IntoArguments, TableMeta, and (subset of) Model
/// This structs are constructed from the *Attribute structs in crate::attr.
#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub table_name: String,
    pub struct_name: Ident,
    pub columns: Vec<ColumnMetadata>,
    pub databases: Vec<String>,

    /// If you're using this, consider whether you should be using a ModelMetadata and its pkey,
    /// which is not optional, instead.
    pub pkey: Option<String>,
}

impl TableMetadata {
    pub fn new(name: &str, columns: Vec<ColumnMetadata>) -> Self {
        TableMetadata {
            table_name: name.to_string(),
            struct_name: Ident(name.to_case(Case::Pascal)),
            pkey: None,
            columns,
            databases: vec![],
        }
    }

    pub fn from_derive(ast: &DeriveInput) -> Result<TableMetadata, SyndecodeError> {
        let mut databases = vec![];
        let struct_name = Ident::from(&ast.ident);
        let mut table_name = None;
        for attr in ast.attrs.iter().filter(|a| a.path().is_ident("ormlite")) {
            let args: ModelAttributes = attr.parse_args().map_err(|e| SyndecodeError(e.to_string()))?;
            if let Some(value) = args.table {
                table_name = Some(value.value());
            }
            if let Some(value) = args.database {
                databases.push(value.value());
            }
        }
        let table_name = table_name.unwrap_or_else(|| struct_name.to_string().to_case(Case::Snake));
        let mut columns = ast
            .fields()
            .map(ColumnMetadata::try_from)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let mut pkey = columns
            .iter()
            .find(|&c| c.marked_primary_key)
            .map(|c| c.clone())
            .map(|c| c.column_name.clone());
        if pkey.is_none() {
            let candidates = sqlmo::util::pkey_column_names(&table_name);
            let c = columns.iter_mut().find(|c| candidates.contains(c.identifier.as_ref()));
            if let Some(c) = c {
                c.has_database_default = true;
                pkey = Some(c.column_name.clone());
            }
        }
        Ok(Self {
            table_name,
            struct_name,
            columns,
            databases,
            pkey,
        })
    }

    pub fn all_fields(&self) -> impl Iterator<Item = &Ident> + '_ {
        self.columns.iter().map(|c| &c.identifier)
    }

    pub fn database_columns(&self) -> impl Iterator<Item = &ColumnMetadata> + '_ {
        self.columns.iter().filter(|&c| !c.skip)
    }

    pub fn many_to_one_joins(&self) -> impl Iterator<Item = &ColumnMetadata> + '_ {
        self.columns.iter().filter(|&c| c.many_to_one_column_name.is_some())
    }
}
