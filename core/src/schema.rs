use std::collections::BTreeMap;
use std::fmt::Formatter;

use std::path::Path;
use anyhow::Result;
use sqlmo::{Schema, Table, schema::Column};
use ormlite_attr::{ColumnMetadata, Ident, InnerType, TableMetadata, TType};
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
                if c.skip {
                    return Ok(None);
                }
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
    fn from_type(ty: &TType) -> Option<Self> {
        use sqlmo::Type::*;
        match ty {
            TType::Vec(v) => {
                if let TType::Inner(p) = v.as_ref() {
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
            TType::Inner(p) => {
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
            TType::Option(o) => {
                let inner = Self::from_type(o)?;
                Some(SqlType {
                    ty: inner.ty,
                    nullable: true,
                })
            }
            TType::Join(_) => {
                None
            }
        }
    }
}

impl TryFromOrmlite for Schema {
    fn try_from_ormlite_project(paths: &[&Path], opts: &Options) -> Result<Self> {
        let mut schema = Self::default();
        let mut fs_schema = schema_from_filepaths(paths, &LoadOptions { verbose: opts.verbose })?;
        let primary_key_type: BTreeMap<String, InnerType> = fs_schema.tables.iter().map(|t|  {
            let pkey = t.primary_key.as_ref()
                .expect(&format!("Could not determine primary key for table {}.", t.table_name));
            let primary_key = t.columns.iter().find(|&c| &c.column_name == pkey)
                .expect(&format!("Could not find primary key column {} for table {}", pkey, t.table_name));
            let pkey_ty = primary_key.column_type.inner_type().clone();
            (t.struct_name.to_string(), pkey_ty)
        }).collect();
        for t in &mut fs_schema.tables {
            for c in &mut t.columns {
                // replace alias types with the real type.
                let inner = c.column_type.inner_type_mut();
                if let Some(f) = fs_schema.type_reprs.get(&inner.ident.0) {
                    inner.ident = Ident(f.clone());
                }
                // replace join types with the primary key type.
                if c.column_type.is_join() {
                    let model_name = c.column_type.inner_type_name();
                    let pkey = primary_key_type.get(&model_name).expect(&format!("Could not find model {} for join", model_name));
                    c.column_type = TType::Inner(pkey.clone());
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
    use ormlite_attr::TType;
    use sqlmo::Type;

    #[test]
    fn test_convert_type() -> Result<()> {

        let s = TType::from(&parse_str::<syn::Path>("String").unwrap());
        assert_matches!(SqlType::from_type(&s).unwrap().ty, Type::Text);
        let s = TType::from(&parse_str::<syn::Path>("u32").unwrap());
        assert_matches!(SqlType::from_type(&s).unwrap().ty, Type::I64);
        let s = TType::from(&parse_str::<syn::Path>("Option<String>").unwrap());
        let s = SqlType::from_type(&s).unwrap();
        assert_matches!(s.ty, Type::Text);
        assert!(s.nullable);
        Ok(())
    }

    #[test]
    fn test_support_vec() {
        let s = TType::from(&parse_str::<syn::Path>("Vec<Uuid>").unwrap());
        let Type::Array(inner) = SqlType::from_type(&s).unwrap().ty else {
            panic!("Expected array");
        };
        assert_eq!(*inner, Type::Uuid);

    }
}
