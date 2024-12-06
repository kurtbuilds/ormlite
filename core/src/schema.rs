use crate::config::Config;
use anyhow::Result as AnyResult;
use ormlite_attr::ModelMeta;
use ormlite_attr::Type;
use ormlite_attr::{schema_from_filepaths, ColumnMeta, Ident, InnerType};
use sqlmo::{schema::Column, Constraint, Schema, Table};
use std::collections::HashMap;
use std::path::Path;

pub fn schema_from_ormlite_project(paths: &[&Path], c: &Config) -> AnyResult<Schema> {
    let mut schema = Schema::default();
    let mut fs_schema = schema_from_filepaths(paths)?;
    let primary_key_type: HashMap<String, InnerType> = fs_schema
        .tables
        .iter()
        .map(|t| {
            let pkey_ty = t.pkey.ty.inner_type().clone();
            (t.ident.to_string(), pkey_ty)
        })
        .collect();
    for t in &mut fs_schema.tables {
        for c in &mut t.table.columns {
            // replace alias types with the real type.
            let inner = c.ty.inner_type_mut();
            if let Some(f) = fs_schema.type_reprs.get(&inner.ident.to_string()) {
                inner.ident = Ident::from(f);
            }
            // replace join types with the primary key type.
            if c.ty.is_join() {
                let model_name = c.ty.inner_type_name();
                let pkey = primary_key_type
                    .get(&model_name)
                    .unwrap_or_else(|| panic!("Could not find model {} for join", model_name));
                c.ty = Type::Inner(pkey.clone());
            }
        }
    }
    for table in fs_schema.tables {
        let table = Table::from_meta(&table);
        schema.tables.push(table);
    }
    let mut table_names: HashMap<String, (String, String)> = schema
        .tables
        .iter()
        .map(|t| (t.name.clone(), (t.name.clone(), t.primary_key().unwrap().name.clone())))
        .collect();
    for (alias, real) in &c.table.aliases {
        let Some(real) = table_names.get(real) else {
            continue;
        };
        table_names.insert(alias.clone(), real.clone());
    }
    for table in &mut schema.tables {
        for column in &mut table.columns {
            if column.primary_key {
                continue;
            }
            if column.name.ends_with("_id") || column.name.ends_with("_uuid") {
                let Some((model_name, _)) = column.name.rsplit_once('_') else {
                    continue;
                };
                if let Some((t, pkey)) = table_names.get(model_name) {
                    let constraint = Constraint::foreign_key(t.to_string(), vec![pkey.clone()]);
                    column.constraint = Some(constraint);
                }
            }
        }
    }
    Ok(schema)
}

#[derive(Debug)]
pub struct Options {
    pub verbose: bool,
}

pub trait FromMeta: Sized {
    type Input;
    fn from_meta(meta: &Self::Input) -> Self;
}

impl FromMeta for Table {
    type Input = ModelMeta;
    fn from_meta(model: &ModelMeta) -> Self {
        let columns = model
            .columns
            .iter()
            .flat_map(|c| {
                if c.skip {
                    return None;
                }
                let mut col = Option::<Column>::from_meta(c)?;
                col.primary_key = model.pkey.name == col.name;
                Some(col)
            })
            .collect();
        Self {
            schema: None,
            name: model.name.clone(),
            columns,
            indexes: vec![],
        }
    }
}

impl FromMeta for Option<Column> {
    type Input = ColumnMeta;
    fn from_meta(meta: &Self::Input) -> Self {
        let mut ty = Nullable::from_type(&meta.ty)?;
        if meta.json {
            ty.ty = sqlmo::Type::Jsonb;
        }
        Some(Column {
            name: meta.name.clone(),
            typ: ty.ty,
            default: None,
            nullable: ty.nullable,
            primary_key: meta.marked_primary_key,
            constraint: None,
        })
    }
}

struct Nullable {
    pub ty: sqlmo::Type,
    pub nullable: bool,
}

impl From<sqlmo::Type> for Nullable {
    fn from(value: sqlmo::Type) -> Self {
        Self {
            ty: value,
            nullable: false,
        }
    }
}

impl Nullable {
    fn from_type(ty: &Type) -> Option<Self> {
        use sqlmo::Type::*;
        match ty {
            Type::Vec(v) => {
                if let Type::Inner(p) = v.as_ref() {
                    if p.ident == "u8" {
                        return Some(Nullable {
                            ty: Bytes,
                            nullable: false,
                        });
                    }
                }
                let v = Self::from_type(v.as_ref())?;
                Some(Nullable {
                    ty: Array(Box::new(v.ty)),
                    nullable: false,
                })
            }
            Type::Inner(p) => {
                let ident = p.ident.to_string();
                let ty = match ident.as_str() {
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
                Some(Nullable { ty, nullable: false })
            }
            Type::Option(o) => {
                let inner = Self::from_type(o)?;
                Some(Nullable {
                    ty: inner.ty,
                    nullable: true,
                })
            }
            Type::Join(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use assert_matches::assert_matches;
    use ormlite_attr::Type;
    use syn::parse_str;

    #[test]
    fn test_convert_type() -> Result<()> {
        use sqlmo::Type as SqlType;
        let s = Type::from(&parse_str::<syn::Path>("String").unwrap());
        assert_matches!(Nullable::from_type(&s).unwrap().ty, SqlType::Text);
        let s = Type::from(&parse_str::<syn::Path>("u32").unwrap());
        assert_matches!(Nullable::from_type(&s).unwrap().ty, SqlType::I64);
        let s = Type::from(&parse_str::<syn::Path>("Option<String>").unwrap());
        let s = Nullable::from_type(&s).unwrap();
        assert_matches!(s.ty, SqlType::Text);
        assert!(s.nullable);
        Ok(())
    }

    #[test]
    fn test_support_vec() {
        use sqlmo::Type as SqlType;
        let s = Type::from(&parse_str::<syn::Path>("Vec<Uuid>").unwrap());
        let SqlType::Array(inner) = Nullable::from_type(&s).unwrap().ty else {
            panic!("Expected array");
        };
        assert_eq!(*inner, SqlType::Uuid);
    }
}
