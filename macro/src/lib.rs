#![allow(unused)]
#![allow(non_snake_case)]

use ormlite_attr::{ColumnAttributes, ColumnMetadata, TableMetadata, TableMetadataBuilder, ColumnMetadataBuilder, ModelAttributes, SyndecodeError, schema_from_filepaths, LoadOptions};
use ormlite_core::config::get_var_model_folders;
use crate::codegen::common::OrmliteCodegen;
use proc_macro::TokenStream;
use std::borrow::Borrow;

use quote::quote;
use lazy_static::lazy_static;
use syn::{Data, DeriveInput, Item, ItemStruct, parse_macro_input};
use ormlite_attr::DeriveInputExt;
use std::collections::HashMap;
use std::ops::Deref;
use once_cell::sync::OnceCell;
use ormlite_core::Error::OrmliteError;

mod codegen;
mod util;

pub(crate) type MetadataCache = HashMap<String, TableMetadata>;

static TABLES: OnceCell<MetadataCache> = OnceCell::new();

fn get_tables() -> &'static MetadataCache {
    TABLES.get_or_init(|| load_metadata_cache())
}

fn load_metadata_cache() -> MetadataCache {
    let mut tables = HashMap::new();
    let paths = get_var_model_folders();
    let paths = paths.iter().map(|p| p.as_path()).collect::<Vec<_>>();
    let schema = schema_from_filepaths(&paths, &LoadOptions::default()).expect("Failed to preload models");
    for meta in schema.tables {
        let name = meta.struct_name.to_string();
        tables.insert(name, meta);
    }
    tables
}

/// For a given struct, determine what codegen to use.
fn get_databases(table_meta: &TableMetadata) -> Vec<Box<dyn OrmliteCodegen>> {
    let mut databases: Vec<Box<dyn OrmliteCodegen>> = Vec::new();
    let dbs = table_meta.databases.clone();
    if dbs.is_empty() {
        #[cfg(feature = "default-sqlite")]
        databases.push(Box::new(codegen::sqlite::SqliteBackend {}));
        #[cfg(feature = "default-postgres")]
        databases.push(Box::new(codegen::postgres::PostgresBackend {}));
        #[cfg(feature = "default-mysql")]
        databases.push(Box::new(codegen::mysql::MysqlBackend {}));
    } else {
        for db in dbs {
            match db.as_str() {
                #[cfg(feature = "sqlite")]
                "sqlite" => databases.push(Box::new(codegen::sqlite::SqliteBackend {})),
                #[cfg(feature = "postgres")]
                "postgres" => databases.push(Box::new(codegen::postgres::PostgresBackend {})),
                #[cfg(feature = "mysql")]
                "mysql" => databases.push(Box::new(codegen::mysql::MysqlBackend {})),
                "sqlite" | "postgres" | "mysql" => panic!("Database {} is not enabled. Enable it with features = [\"{}\"]", db, db),
                _ => panic!("Unknown database: {}", db),
            }
        }
    }
    if databases.is_empty() {
        let mut count = 0;
        #[cfg(feature = "sqlite")]
        {
            count += 1;
        }
        #[cfg(feature = "postgres")]
        {
            count += 1;
        }
        #[cfg(feature = "mysql")]
        {
            count += 1;
        }
        if count > 1 {
            panic!("You have more than one database configured using features, but no database is specified for this model. \
            Specify a database for the model like this:\n\n#[ormlite(database = \"<db>\")]\n\nOr you can enable \
            a default database feature:\n\n # Cargo.toml\normlite = {{ features = [\"default-<db>\"] }}");
        }
    }
    if databases.is_empty() {
        #[cfg(feature = "sqlite")]
        databases.push(Box::new(codegen::sqlite::SqliteBackend {}));
        #[cfg(feature = "postgres")]
        databases.push(Box::new(codegen::postgres::PostgresBackend {}));
        #[cfg(feature = "mysql")]
        databases.push(Box::new(codegen::mysql::MysqlBackend {}));
    }
    databases
}

/// Derive macro for `#[derive(Model)]` It additionally generates FromRow for the struct, since
/// Model requires FromRow.
#[proc_macro_derive(Model, attributes(ormlite))]
pub fn expand_ormlite_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let Data::Struct(data) = &ast.data else { panic!("Only structs can derive Model"); };

    let table_meta = TableMetadata::try_from(&ast).expect("Failed to parse table metadata");
    if table_meta.primary_key.is_none() {
        panic!("No column marked with #[ormlite(primary_key)], and no column named id, uuid, {0}_id, or {0}_uuid", table_meta.table_name);
    }

    let mut databases = get_databases(&table_meta);
    let tables = get_tables();

    let first = databases.remove(0);

    let primary = {
        let db = first;
        let impl_TableMeta = db.impl_TableMeta(&table_meta);
        let static_join_descriptions = db.static_join_descriptions(&ast, &table_meta, &tables);
        let impl_Model = db.impl_Model(&ast, &table_meta, tables);
        let impl_FromRow = db.impl_FromRow(&ast, &table_meta, &tables);
        let impl_from_row_using_aliases = db.impl_from_row_using_aliases(&ast, &table_meta, &tables);

        let struct_ModelBuilder = db.struct_ModelBuilder(&ast, &table_meta);
        let impl_ModelBuilder = db.impl_ModelBuilder(&ast, &table_meta);

        let struct_InsertModel = db.struct_InsertModel(&ast, &table_meta);
        let impl_InsertModel = db.impl_InsertModel(&ast, &table_meta);

        quote! {
            #impl_TableMeta
            #static_join_descriptions
            #impl_Model
            #impl_FromRow
            #impl_from_row_using_aliases

            #struct_ModelBuilder
            #impl_ModelBuilder

            #struct_InsertModel
            #impl_InsertModel
        }
    };

    let rest = databases.iter().map(|db| {
        let impl_Model = db.impl_Model(&ast, &table_meta, tables);
        // let impl_FromRow = db.impl_FromRow(&ast, &table_meta, &tables);
        // TODO This should be in there, but we need to turn it into a trait tahts' generic on DB
        // instead of being a method on impl #model
        // let impl_from_row_using_aliases = db.impl_from_row_using_aliases(&ast, &table_meta, &tables);
        quote! {
            #impl_Model
            // #impl_FromRow
            // #impl_from_row_using_aliases
        }
    });

    TokenStream::from(quote! {
        #primary
        #(#rest)*
    })
}

#[proc_macro_derive(FromRow, attributes(ormlite))]
pub fn expand_derive_fromrow(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let Data::Struct(data) = &ast.data else { panic!("Only structs can derive Model"); };

    let table_meta = TableMetadata::try_from(&ast).unwrap();

    let databases = get_databases(&table_meta);
    let tables = get_tables();

    let expanded = databases.iter().map(|db| {
        let impl_FromRow = db.impl_FromRow(&ast, &table_meta, &tables);
        let impl_from_row_using_aliases = db.impl_from_row_using_aliases(&ast, &table_meta, &tables);
        quote! {
            #impl_FromRow
            #impl_from_row_using_aliases
        }
    });

    TokenStream::from(quote! {
        #(#expanded)*
    })
}

#[proc_macro_derive(TableMeta, attributes(ormlite))]
pub fn expand_derive_table_meta(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let Data::Struct(data) = &ast.data else { panic!("Only structs can derive Model"); };

    let table_meta = TableMetadata::try_from(&ast).expect("Failed to parse table metadata");
    let databases = get_databases(&table_meta);
    let db = databases.first().expect("No database configured");

    let impl_TableMeta = db.impl_TableMeta(&table_meta);
    TokenStream::from(impl_TableMeta)
}

#[proc_macro_derive(IntoArguments, attributes(ormlite))]
pub fn expand_derive_into_arguments(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let Data::Struct(data) = &ast.data else { panic!("Only structs can derive Model"); };

    let table_meta = TableMetadata::try_from(&ast).unwrap();
    let databases = get_databases(&table_meta);

    let expanded = databases.iter().map(|db| {
        let impl_IntoArguments = db.impl_IntoArguments(&table_meta);
        impl_IntoArguments
    });
    TokenStream::from(quote! {
        #(#expanded)*
    })
}
