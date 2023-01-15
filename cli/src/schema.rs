
use std::fmt::Formatter;

use std::path::Path;
use anyhow::Result;
use sqldiff::Schema;
use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments, Type};
use ormlite_attr::{ColumnMetadata, TableMetadata};
use sqldiff::Table;
use syn::__private::ToTokens;
use ormlite_attr::{load_from_project, LoadOptions};
use crate::command::Migrate;

pub trait TryFromOrmlite: Sized {
    fn try_from_ormlite_project(path: &[&Path], opts: &Migrate) -> Result<Self>;
}

trait SqlDiffTableExt {
    fn from_metadata(metadata: &TableMetadata) -> Result<Self, TypeTranslationError> where Self: Sized;
}

impl SqlDiffTableExt for Table {
    fn from_metadata(metadata: &TableMetadata) -> Result<Self, TypeTranslationError> {
        Ok(Self {
            name: metadata.table_name.clone(),
            columns: metadata.columns.iter().filter(|c| !c.is_join()).map(|c| {
                let mut col = sqldiff::Column::from_metadata(c)?;
                col.primary_key = metadata.primary_key.as_ref().map(|c| c == col.name.as_str()).unwrap_or(false);
                Ok(col)
            }).collect::<Result<Vec<_>,_>>()?,
            indexes: vec![],
        })
    }
}

trait SqlDiffColumnExt {
    fn from_metadata(metadata: &ColumnMetadata) -> Result<sqldiff::Column, TypeTranslationError>;
}

impl SqlDiffColumnExt for sqldiff::Column {
    fn from_metadata(metadata: &ColumnMetadata) -> Result<sqldiff::Column, TypeTranslationError> {
        let ty = SqlType::from_type(&metadata.column_type)?;
        Ok(Self {
            name: metadata.column_name.clone(),
            typ: ty.ty,
            default: None,
            nullable: ty.nullable,
            primary_key: metadata.marked_primary_key,
        })
    }
}

struct SqlType {
    pub ty: sqldiff::Type,
    pub nullable: bool,
}

impl From<sqldiff::Type> for SqlType {
    fn from(value: sqldiff::Type) -> Self {
        Self {
            ty: value,
            nullable: false,
        }
    }
}

#[derive(Debug)]
pub struct TypeTranslationError(pub String);

impl std::error::Error for TypeTranslationError {}

impl std::fmt::Display for TypeTranslationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not translate type: {}", self.0)
    }
}

impl SqlType {
    fn from_type(ty: &Type) -> Result<Self, TypeTranslationError> {
        use sqldiff::Type::*;
        match ty {
            Type::Paren(t) => {
                Self::from_type(&t.elem)
            }
            Type::Path(s) => {
                let segment = s.path.segments.last().expect("No segments in path");
                let ty = match segment.ident.to_string().as_str() {
                    "String" => Text,
                    "u32" => Integer,
                    "u64" => Integer,
                    "i32" => Integer,
                    "i64" => Integer,
                    "f32" => Numeric,
                    "f64" => Numeric,
                    "bool" => Boolean,
                    "Uuid" => Uuid,
                    "Json" => Jsonb,
                    "DateTime" => DateTime,
                    "Option" => {
                        let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &segment.arguments else { panic!("No args in path") };
                        let arg = args.first().expect("No arguments in angle brackets.");
                        let GenericArgument::Type(arg) = arg else { panic!("No type in argument") };
                        let inner = Self::from_type(arg)?;
                        return Ok(SqlType {
                            ty: inner.ty,
                            nullable: true,
                        })
                    },
                    "Vec" => {
                        let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &segment.arguments else { panic!("No args in path") };
                        let arg = args.first().expect("No arguments in angle brackets.");
                        let GenericArgument::Type(arg) = arg else { panic!("No type in argument") };
                        if arg.to_token_stream().to_string() != "u8" {
                            return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {}", ty.to_token_stream())));
                        }
                        Bytes
                    }
                    _ => return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {}", ty.to_token_stream())))
                };
                Ok(SqlType {
                    ty,
                    nullable: false,
                })
            }
            _ => return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {}", ty.to_token_stream())))
        }
    }
}

impl TryFromOrmlite for Schema {
    fn try_from_ormlite_project(paths: &[&Path], opts: &Migrate) -> Result<Self> {
        let mut schema = Self::new();
        let tables = load_from_project(paths, &LoadOptions { verbose: opts.verbose })?;
        for table in tables {
            let table = Table::from_metadata(&table)?;
            schema.tables.push(table);
        }
        Ok(schema)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_str};
    use assert_matches::assert_matches;

    #[test]
    fn test_convert_type() -> Result<()> {
        use sqldiff::Type;

        let s = parse_str::<syn::Type>("String").unwrap();
        assert_matches!(SqlType::from_type(&s)?.ty, Type::Text);
        let s = parse_str::<syn::Type>("u32").unwrap();
        assert_matches!(SqlType::from_type(&s)?.ty, Type::Integer);
        let s = parse_str::<syn::Type>("Option<String>").unwrap();
        let s = SqlType::from_type(&s)?;
        assert_matches!(s.ty, Type::Text);
        assert!(s.nullable);
        Ok(())
    }
}