use derive_builder::Builder;
use syn::{DeriveInput, Field, Type};
use crate::{ColumnAttributes, ModelAttributes, SyndecodeError};
use crate::DeriveInputExt;

/// All the metadata we can capture about a table
#[derive(Builder, Debug)]
#[builder(field(public))]
pub struct TableMetadata {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Vec<ColumnMetadata>,
    pub insert_struct: Option<String>,
}

impl TableMetadata {
    pub fn builder() -> TableMetadataBuilder {
        TableMetadataBuilder::default()
    }

    pub fn builder_from_struct_attributes(ast: &DeriveInput) -> Result<TableMetadataBuilder, SyndecodeError> {
        let mut builder = TableMetadata::builder();
        builder.insert_struct(None);
        for attr in ast.attrs.iter().filter(|a| a.path.is_ident("ormlite")) {
            let args: ModelAttributes = attr.parse_args()
                .map_err(|e| SyndecodeError(e.to_string()))?;
            if let Some(value) = args.table {
                builder.table_name(value.value());
            }
            if let Some(value) = args.Insertable {
                builder.insert_struct(Some(value.to_string()));
            }
        }
        Ok(builder)
    }
}


impl TableMetadataBuilder {
    pub fn complete_with_struct_body(&mut self, ast: &DeriveInput) -> Result<TableMetadata, SyndecodeError> {
        let model = &ast.ident;
        let model_lowercased = model.to_string().to_lowercase();
        self.table_name.get_or_insert(model_lowercased.clone());

        let mut cols = ast.fields()
            .map(|f| ColumnMetadata::try_from(f))
            .collect::<Result<Vec<_>,_>>().unwrap();
        let mut primary_key = cols
            .iter()
            .filter(|c| c.marked_primary_key)
            .map(|m| m.column_name.clone())
            .next();
        if primary_key.is_none() {
            for f in cols.iter_mut() {
                if [
                    "id".to_string(),
                    "uuid".to_string(),
                    format!("{}_id", self.table_name.as_ref().unwrap()),
                    format!("{}_uuid", self.table_name.as_ref().unwrap()),
                ]
                    .contains(&f.column_name)
                {
                    primary_key = Some(f.column_name.clone());
                    f.has_database_default = true;
                    break;
                }
            }
        }
        if primary_key.is_none() {
            panic!("No column marked with #[ormlite(primary_key)], and no column named id, uuid, {0}_id, or {0}_uuid", model_lowercased);
        } else {
            self.primary_key(primary_key.unwrap());
        }
        self.columns(cols);
        self.build().map_err(|e| SyndecodeError(e.to_string()))
    }
}


impl TryFrom<&DeriveInput> for TableMetadata {
    type Error = SyndecodeError;

    fn try_from(ast: &DeriveInput) -> Result<Self, Self::Error> {
        TableMetadata::builder_from_struct_attributes(&ast)?
            .complete_with_struct_body(&ast)
    }
}


/// All the metadata we can capture about a column
#[derive(Clone, Debug, Builder)]
pub struct ColumnMetadata {
    pub column_name: String,
    pub column_type: Type,
    pub marked_primary_key: bool,
    pub has_database_default: bool,
}

impl ColumnMetadata {
    pub fn builder() -> ColumnMetadataBuilder {
        ColumnMetadataBuilder::default()
    }
}

impl TryFrom<&Field> for ColumnMetadata {
    type Error = SyndecodeError;

    fn try_from(f: &Field) -> Result<Self, Self::Error> {
        let mut builder = ColumnMetadata::builder();
        builder
            .column_name(f.ident.as_ref().unwrap().to_string())
            .column_type(f.ty.clone())
            .marked_primary_key(false)
            .has_database_default(false);
        for attr in f.attrs.iter().filter(|a| a.path.is_ident("ormlite")) {
            let args: ColumnAttributes = attr.parse_args().unwrap();
            if args.primary_key {
                builder.marked_primary_key(true);
                builder.has_database_default(true);
            }
            if args.default {
                builder.has_database_default(true);
            }
        }
        builder.build().map_err(|e| SyndecodeError(e.to_string()))
    }
}