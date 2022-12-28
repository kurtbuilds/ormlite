/// Decode a sqldiff::Schema from the current code base.
use std::collections::HashMap;
use std::fmt::Formatter;
use std::fs;
use std::path::Path;
use anyhow::Result;
use ignore::Walk;
use sqldiff::Schema;
use syn::{AngleBracketedGenericArguments, DeriveInput, GenericArgument, Item, PathArguments, Type};
use ormlite_attr::{ColumnMetadata, SyndecodeError, TableMetadata};
use sqldiff::Table;
use syn::__private::ToTokens;

use crate::syndecode::Attributes;

pub trait TryFromOrmlite: Sized {
    fn try_from_ormlite_project(path: &Path) -> Result<Self>;
}

trait SqlDiffTableExt {
    fn from_metadata(metadata: &TableMetadata) -> Result<Self, TypeTranslationError> where Self: Sized;
}

impl SqlDiffTableExt for Table {
    fn from_metadata(metadata: &TableMetadata) -> Result<Self, TypeTranslationError> {
        Ok(Self {
            name: metadata.table_name.clone(),
            columns: metadata.columns.iter().map(|c| {
                let mut col = sqldiff::Column::from_metadata(c)?;
                col.primary_key = metadata.primary_key == col.name;
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
        println!("from_type: {:?}", ty);
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
                            return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {}", ty.to_token_stream().to_string())));
                        }
                        Bytes
                    }
                    _ => return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {}", ty.to_token_stream().to_string())))
                };
                return Ok(SqlType {
                    ty,
                    nullable: false,
                })
            }
            _ => return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {}", ty.to_token_stream().to_string())))
        }
    }
}

impl TryFromOrmlite for Schema {
    fn try_from_ormlite_project(path: &Path) -> Result<Self> {
        let walk = Walk::new(path)
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|e| e == "rs")
                .unwrap_or(false));

        let mut schema = Self::new();

        for entry in walk {
            let contents = fs::read_to_string(&entry.path())?;
            if !contents.contains("Model") {
                continue;
            }
            let mut ast = syn::parse_file(&contents)?;
            let structs = ast.items.into_iter().filter_map(|item| match item {
                Item::Struct(s) => Some(s),
                _ => None,
            })
                .map(|s| {
                    let attrs = Attributes::from(&s.attrs);
                    (s, attrs)
                })
                .filter(|(_, attrs)| attrs.derives("Model"))
                .collect::<Vec<_>>();
            for (item, attrs) in structs {
                let derive: DeriveInput = item.into();
                let table = TableMetadata::try_from(&derive)
                    .map_err(|e| SyndecodeError(format!(
                        "{}: Encounterd an error while scanning for #[derive(Model)] structs: {}",
                        entry.path().display(), e.to_string()))
                    )?;
                let table = sqldiff::Table::from_metadata(&table)?;
                schema.tables.push(table);
            }
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
    fn test_convert_type() {
        use sqldiff::Type;

        let s = parse_str::<syn::Type>("String").unwrap();
        assert_matches!(SqlType::from_type(&s).ty, Type::Text);
        let s = parse_str::<syn::Type>("u32").unwrap();
        assert_matches!(SqlType::from_type(&s).ty, Type::Integer);
        let s = parse_str::<syn::Type>("Option<String>").unwrap();
        let s = SqlType::from_type(&s);
        assert_matches!(s.ty, Type::Text);
        assert!(s.nullable);
    }
}