
use std::fmt::Formatter;

use std::path::Path;
use anyhow::Result;
use sqlmo::{Schema, Table, schema::Column};
use ormlite_attr::{ColumnMetadata, TableMetadata};
use ormlite_attr::{load_from_project, LoadOptions};

#[derive(Debug)]
pub struct Options {
    pub verbose: bool
}

pub trait TryFromOrmlite: Sized {
    fn try_from_ormlite_project(path: &[&Path], opts: &Options) -> Result<Self>;
}

trait SqlDiffTableExt {
    fn from_metadata(metadata: &TableMetadata) -> Result<Self, TypeTranslationError> where Self: Sized;
}

impl SqlDiffTableExt for Table {
    fn from_metadata(metadata: &TableMetadata) -> Result<Self, TypeTranslationError> {
        Ok(Self {
            schema: None,
            name: metadata.table_name.clone(),
            columns: metadata.columns.iter().map(|c| {
                let Some(mut col) = Column::from_metadata(c)? else {
                    return Ok(None);
                };
                col.primary_key = metadata.primary_key.as_ref().map(|c| c == col.name.as_str()).unwrap_or(false);
                Ok(Some(col))
            })
                .filter_map(|c| c.transpose())
                .collect::<Result<Vec<_>,_>>()?,
            indexes: vec![],
        })
    }
}

trait SqlDiffColumnExt {
    fn from_metadata(metadata: &ColumnMetadata) -> Result<Option<Column>, TypeTranslationError>;
}

impl SqlDiffColumnExt for Column {
    fn from_metadata(metadata: &ColumnMetadata) -> Result<Option<Column>, TypeTranslationError> {
        let Some(ty) = SqlType::from_type(&metadata.column_type)? else {
            return Ok(None)
        };
        Ok(Some(Self {
            name: metadata.column_name.clone(),
            typ: ty.ty,
            default: None,
            nullable: ty.nullable,
            primary_key: metadata.marked_primary_key,
        }))
    }
}

struct SqlType {
    pub ty: sqlmo::Type,
    pub nullable: bool,
}

impl From<sqlmo::Type> for SqlType {
    fn from(value: sqlmo::Type) -> Self {
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
    fn from_type(ty: &ormlite_attr::Type) -> Result<Option<Self>, TypeTranslationError> {
        use sqlmo::Type::*;
        use ormlite_attr::Type;
        match ty {
            Type::Vec(v) => {
                let Type::Primitive(p) = v.as_ref() else {
                    return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {:?}", ty)))
                };
                if p.ident.0 != "u8" {
                    return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {:?}", ty)))
                }
                Ok(Some(SqlType {
                    ty: Bytes,
                    nullable: false,
                }))
            }
            Type::Foreign(_) => {
                Ok(Some(SqlType {
                    ty: Jsonb,
                    nullable: false,
                }))
            }
            Type::Primitive(p) => {
                let ident = p.ident.0.as_str();
                let ty = match ident {
                    "String" => Text,
                    "u8" => SmallInt,
                    "u32" => Integer,
                    "u64" => Integer,
                    "i32" => Integer,
                    "i64" => Integer,
                    "f32" => Float64,
                    "f64" => Float64,
                    "bool" => Boolean,
                    "Uuid" => Uuid,
                    "Json" => Jsonb,
                    "DateTime" => DateTime,
                    _ => return Err(TypeTranslationError(format!("Don't know how to convert Rust type to SQL: {:?}", ty)))
                };
                Ok(Some(SqlType {
                    ty,
                    nullable: false,
                }))
            }
            Type::Option(o) => {
                let inner = Self::from_type(o)?.unwrap();
                Ok(Some(SqlType {
                    ty: inner.ty,
                    nullable: true,
                }))
            }
            Type::Join(_) => {
                Ok(None)
            }
        }
    }
}

impl TryFromOrmlite for Schema {
    fn try_from_ormlite_project(paths: &[&Path], opts: &Options) -> Result<Self> {
        let mut schema = Self::default();
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
        use sqlmo::Type;

        let s = ormlite_attr::Type::from(&parse_str::<syn::Path>("String").unwrap());
        assert_matches!(SqlType::from_type(&s)?.unwrap().ty, Type::Text);
        let s = ormlite_attr::Type::from(&parse_str::<syn::Path>("u32").unwrap());
        assert_matches!(SqlType::from_type(&s)?.unwrap().ty, Type::Integer);
        let s = ormlite_attr::Type::from(&parse_str::<syn::Path>("Option<String>").unwrap());
        let s = SqlType::from_type(&s)?.unwrap();
        assert_matches!(s.ty, Type::Text);
        assert!(s.nullable);
        Ok(())
    }
}