use derive_builder::Builder;
use syn::Type;

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
}

/// All the metadata we can capture about a column
#[derive(Clone, Debug, Builder)]
#[builder(field(public))]
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