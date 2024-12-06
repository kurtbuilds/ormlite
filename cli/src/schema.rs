use crate::config::Config;
use anyhow::Result as AnyResult;
use ormlite_attr::{schema_from_filepaths, Ident, InnerType, Type};
use ormlite_core::schema::FromMeta;
use sqlmo::{Constraint, Schema, Table};
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

