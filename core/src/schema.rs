use std::fmt::Formatter;

use std::path::Path;
use anyhow::Result;
use sqlmo::{Schema, Table, schema::Column};
use ormlite_attr::{ColumnMetadata, Ident, TableMetadata};
use ormlite_attr::{schema_from_filepaths, LoadOptions};

#[derive(Debug)]
pub struct Options {
    pub verbose: bool,
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
                .collect::<Result<Vec<_>, _>>()?,
            indexes: vec![],
        })
    }
}

trait SqlDiffColumnExt {
    fn from_metadata(metadata: &ColumnMetadata) -> Result<Option<Column>, TypeTranslationError>;
}

impl SqlDiffColumnExt for Column {
    fn from_metadata(metadata: &ColumnMetadata) -> Result<Option<Column>, TypeTranslationError> {
        let Some(ty) = SqlType::from_type(&metadata.column_type) else {
            return Ok(None);
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
    fn from_type(ty: &ormlite_attr::Type) -> Option<Self> {
        use sqlmo::Type::*;
        use ormlite_attr::Type;
        match ty {
            Type::Vec(v) => {
                if let Type::Inner(p) = v.as_ref() {
                    if p.ident.0 == "u8" {
                        return Some(SqlType {
                            ty: Bytes,
                            nullable: false,
                        });
                    }
                }
                let v = Self::from_type(v.as_ref())?;
                Some(SqlType {
                    ty: Array(Box::new(v.ty)),
                    nullable: true,
                })
            }
            Type::Inner(p) => {
                let ident = p.ident.0.as_str();
                let ty = match ident {
                    // signed
                    "i8" => I16,
                    "i16" => I16,
                    "i32" => I32,
                    "i64" => I64,
                    "i128" => Decimal,
                    "isize" => I64,
                    // unsigned
                    "u8" => I16,
                    "u16" => I32,
                    "u32" => I64,
                    // Turns out postgres doesn't support u64.
                    "u64" => Decimal,
                    "u128" => Decimal,
                    "usize" => Decimal,
                    // float
                    "f32" => F32,
                    "f64" => F64,
                    // bool
                    "bool" => Boolean,
                    // string
                    "String" => Text,
                    "str" => Text,
                    // date
                    "DateTime" => DateTime,
                    "NaiveDate" => Date,
                    "NaiveTime" => DateTime,
                    "NaiveDateTime" => DateTime,
                    // decimal
                    "Decimal" => Decimal,
                    // uuid
                    "Uuid" => Uuid,
                    // json
                    "Json" => Jsonb,
                    z => Other(z.to_string()),
                };
                Some(SqlType {
                    ty,
                    nullable: false,
                })
            }
            Type::Option(o) => {
                let inner = Self::from_type(o)?;
                Some(SqlType {
                    ty: inner.ty,
                    nullable: true,
                })
            }
            Type::Join(_) => {
                None
            }
        }
    }
}

impl TryFromOrmlite for Schema {
    fn try_from_ormlite_project(paths: &[&Path], opts: &Options) -> Result<Self> {
        let mut schema = Self::default();
        let mut fs_schema = schema_from_filepaths(paths, &LoadOptions { verbose: opts.verbose })?;
        for t in &mut fs_schema.tables {
            for c in &mut t.columns {
                let inner = c.column_type.inner_type_mut();
                if let Some(f) = fs_schema.type_reprs.get(&inner.ident.0) {
                    inner.ident = Ident(f.clone());
                }
            }

        }
        for table in fs_schema.tables {
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
        assert_matches!(SqlType::from_type(&s).unwrap().ty, Type::Text);
        let s = ormlite_attr::Type::from(&parse_str::<syn::Path>("u32").unwrap());
        assert_matches!(SqlType::from_type(&s).unwrap().ty, Type::I64);
        let s = ormlite_attr::Type::from(&parse_str::<syn::Path>("Option<String>").unwrap());
        let s = SqlType::from_type(&s).unwrap();
        assert_matches!(s.ty, Type::Text);
        assert!(s.nullable);
        Ok(())
    }
}