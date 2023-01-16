use derive_builder::Builder;
use syn::{DeriveInput, Field, Type};
use crate::{ColumnAttributes, ModelAttributes, SyndecodeError};
use crate::DeriveInputExt;
use convert_case::{Case, Casing};

/// All the metadata we can capture about a table
#[derive(Builder, Debug)]
pub struct TableMetadata {
    pub table_name: String,
    pub struct_name: syn::Ident,
    pub primary_key: Option<String>,
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
        builder.struct_name(ast.ident.clone());
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

    pub fn all_fields(&self, span: proc_macro2::Span) -> impl Iterator<Item=syn::Ident> + '_ {
        self.columns.iter()
            .map(move |c| syn::Ident::new(&c.column_name, span))
    }
}



impl TableMetadataBuilder {
    pub fn complete_with_struct_body(&mut self, ast: &DeriveInput) -> Result<TableMetadata, SyndecodeError> {
        let model = &ast.ident;
        let model_lowercased = model.to_string().to_case(Case::Snake);
        self.table_name.get_or_insert(model_lowercased);

        let mut cols = ast.fields()
            .map(ColumnMetadata::try_from)
            .collect::<Result<Vec<_>, _>>().unwrap();
        let mut primary_key = cols
            .iter()
            .filter(|c| c.marked_primary_key)
            .map(|c| c.column_name.clone())
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
        self.primary_key(primary_key);
        self.columns(cols);
        self.build().map_err(|e| SyndecodeError(e.to_string()))
    }
}


impl TryFrom<&DeriveInput> for TableMetadata {
    type Error = SyndecodeError;

    fn try_from(ast: &DeriveInput) -> Result<Self, Self::Error> {
        TableMetadata::builder_from_struct_attributes(ast)?
            .complete_with_struct_body(ast)
    }
}


/// All the metadata we can capture about a column
#[derive(Clone, Debug, Builder)]
pub struct ColumnMetadata {
    /// Name of the column in the database
    pub column_name: String,
    pub column_type: Type,
    /// Only says whether the primary key is marked (with an attribute). Use table_metadata.primary_key to definitively know the primary key.
    pub marked_primary_key: bool,
    pub has_database_default: bool,
    /// Identifier used in Rust to refer to the column
    pub identifier: syn::Ident,

    // only for joins
    pub many_to_one_key: Option<syn::Ident>,
    pub many_to_many_table: Option<syn::Path>,
    pub one_to_many_foreign_key: Option<syn::Path>,
}

impl ColumnMetadata {
    pub fn builder() -> ColumnMetadataBuilder {
        ColumnMetadataBuilder::default()
    }

    pub fn is_join(&self) -> bool {
        ty_is_join(&self.column_type)
    }

    /// We expect this to only return a `Model` of some kind.
    pub fn joined_struct_name(&self) -> Option<String> {
        let Some(path) = self.joined_path() else {
            return None;
        };
        let Some(segment) = path.segments.last() else {
            return None;
        };
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return Some(segment.ident.to_string());
        };
        let Some(arg) = args.args.last() else {
            return None;
        };
        let syn::GenericArgument::Type(Type::Path(path)) = arg else {
            return None;
        };
        path.path.segments.last().map(|s| s.ident.to_string())
    }

    /// Whatever is inside the `Join`. We're expecting a `Model` or a `Vec<Model>`.
    pub fn joined_path(&self) -> Option<&syn::Path> {
        let Type::Path(path) = &self.column_type else {
            return None;
        };
        let Some(segment) = path.path.segments.last() else {
            return None;
        };
        if segment.ident != "Join" {
            return None;
        }
        // go inside Join<...>
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return None;
        };
        let Some(arg) = args.args.last() else {
            return None;
        };
        let syn::GenericArgument::Type(Type::Path(path)) = arg else {
            return None;
        };
        Some(&path.path)
    }
}


impl TryFrom<&Field> for ColumnMetadata {
    type Error = SyndecodeError;

    fn try_from(f: &Field) -> Result<Self, Self::Error> {
        let mut builder = ColumnMetadata::builder();
        let ident = f.ident.as_ref().expect("No ident on field");
        builder
            .column_name(ident.to_string())
            .identifier(ident.clone())
            .column_type(f.ty.clone())
            .marked_primary_key(false)
            .has_database_default(false)
            .many_to_one_key(None)
            .many_to_many_table(None)
            .one_to_many_foreign_key(None)
        ;
        let mut has_join_directive = false;
        for attr in f.attrs.iter().filter(|a| a.path.is_ident("ormlite")) {
            let args: ColumnAttributes = attr.parse_args().unwrap();
            if args.primary_key {
                builder.marked_primary_key(true);
                builder.has_database_default(true);
            }
            if args.default {
                builder.has_database_default(true);
            }

            if let Some(column_name) = args.column {
                builder.column_name(column_name.value());
            }
            if let Some(value) = args.many_to_one_key {
                let ident = value.segments.last().unwrap().ident.clone();
                builder.many_to_one_key(Some(ident));
                has_join_directive = true;
            }
            if let Some(value) = args.many_to_many_table {
                builder.many_to_many_table(Some(value));
                has_join_directive = true;
            }
            if let Some(value) = args.one_to_many_foreign_key {
                builder.one_to_many_foreign_key(Some(value));
                has_join_directive = true;
            }
        }
        if ty_is_join(&f.ty) && !has_join_directive {
            return Err(SyndecodeError(format!("Column {ident} is a Join. You must specify one of these attributes: many_to_one_key, many_to_many_table_name, or one_to_many_foreign_key")));
        }
        builder.build().map_err(|e| SyndecodeError(e.to_string()))
    }
}

/// bool whether the given type is `Join`
fn ty_is_join(ty: &Type) -> bool {
    let p = match ty {
        Type::Path(p) => p,
        _ => return false,
    };
    p.path.segments.last().map(|s| s.ident == "Join").unwrap_or(false)
}
