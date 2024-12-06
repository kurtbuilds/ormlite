#![allow(unused)]
#![allow(non_snake_case)]

use codegen::insert::impl_Insert;
use ormlite_attr::InsertMeta;
use proc_macro::TokenStream;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::env::var;
use std::ops::Deref;

use once_cell::sync::OnceCell;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

use codegen::into_arguments::impl_IntoArguments;
use ormlite_attr::schema_from_filepaths;
use ormlite_attr::DeriveInputExt;
use ormlite_attr::ModelMeta;
use ormlite_attr::TableMeta;
use ormlite_core::config::get_var_model_folders;

use crate::codegen::common::OrmliteCodegen;
use crate::codegen::from_row::{impl_FromRow, impl_from_row_using_aliases};
use crate::codegen::insert::impl_InsertModel;
use crate::codegen::insert_model::struct_InsertModel;
use crate::codegen::join_description::static_join_descriptions;
use crate::codegen::meta::{impl_JoinMeta, impl_TableMeta};
use crate::codegen::model::impl_Model;
use crate::codegen::model_builder::{impl_ModelBuilder, struct_ModelBuilder};

mod codegen;
mod util;

/// Mapping from StructName -> ModelMeta
pub(crate) type MetadataCache = HashMap<String, ModelMeta>;

static TABLES: OnceCell<MetadataCache> = OnceCell::new();

fn get_tables() -> &'static MetadataCache {
    fn fun_name() -> HashMap<String, ModelMeta> {
        load_metadata_cache()
    }
    TABLES.get_or_init(fun_name)
}

fn load_metadata_cache() -> MetadataCache {
    let mut tables = HashMap::new();
    let paths = get_var_model_folders();
    let paths = paths.iter().map(|p| p.as_path()).collect::<Vec<_>>();
    let schema = schema_from_filepaths(&paths).expect("Failed to preload models");
    for meta in schema.tables {
        let name = meta.ident.to_string();
        tables.insert(name, meta);
    }
    tables
}

/// For a given struct, determine what codegen to use.
fn get_databases(table_meta: &TableMeta) -> Vec<Box<dyn OrmliteCodegen>> {
    let mut databases: Vec<Box<dyn OrmliteCodegen>> = Vec::new();
    let dbs = table_meta.databases.clone();
    if dbs.is_empty() {
        #[cfg(feature = "default-sqlite")]
        databases.push(Box::new(codegen::sqlite::SqliteBackend {}));
        #[cfg(feature = "default-postgres")]
        databases.push(Box::new(codegen::postgres::PostgresBackend));
        #[cfg(feature = "default-mysql")]
        databases.push(Box::new(codegen::mysql::MysqlBackend {}));
    } else {
        for db in dbs {
            match db.as_str() {
                #[cfg(feature = "sqlite")]
                "sqlite" => databases.push(Box::new(codegen::sqlite::SqliteBackend {})),
                #[cfg(feature = "postgres")]
                "postgres" => databases.push(Box::new(codegen::postgres::PostgresBackend)),
                #[cfg(feature = "mysql")]
                "mysql" => databases.push(Box::new(codegen::mysql::MysqlBackend {})),
                "sqlite" | "postgres" | "mysql" => {
                    panic!("Database {} is not enabled. Enable it with features = [\"{}\"]", db, db)
                }
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
        databases.push(Box::new(codegen::postgres::PostgresBackend));
        #[cfg(feature = "mysql")]
        databases.push(Box::new(codegen::mysql::MysqlBackend {}));
    }
    if databases.is_empty() {
        panic!(
            r#"No database is enabled. Enable one of these features for the ormlite crate: postgres, mysql, sqlite"#
        );
    }
    databases
}

/// Derive macro for `#[derive(Model)]` It additionally generates FromRow for the struct, since
/// Model requires FromRow.
#[proc_macro_derive(Model, attributes(ormlite))]
pub fn expand_ormlite_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let meta = ModelMeta::from_derive(&ast);
    let mut databases = get_databases(&meta.table);
    let tables = get_tables();
    let first = databases.remove(0);

    let primary = {
        let db = first.as_ref();
        let impl_TableMeta = impl_TableMeta(&meta.table, Some(meta.pkey.name.as_str()));
        let impl_JoinMeta = impl_JoinMeta(&meta);
        let static_join_descriptions = static_join_descriptions(&meta.table, tables);
        let impl_Model = impl_Model(db, &meta, tables);
        let impl_FromRow = impl_FromRow(db, &meta.table, tables);
        let impl_from_row_using_aliases = impl_from_row_using_aliases(db, &meta.table, tables);

        let struct_ModelBuilder = struct_ModelBuilder(&ast, &meta);
        let impl_ModelBuilder = impl_ModelBuilder(db, &meta);

        let struct_InsertModel = struct_InsertModel(&ast, &meta);
        let impl_InsertModel = impl_InsertModel(db, &meta);

        quote! {
            #impl_TableMeta
            #impl_JoinMeta

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
        let impl_Model = impl_Model(db.as_ref(), &meta, tables);
        quote! {
            #impl_Model
        }
    });

    TokenStream::from(quote! {
        #primary
        #(#rest)*
    })
}

#[proc_macro_derive(Insert, attributes(ormlite))]
pub fn expand_ormlite_insert(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let mut meta = InsertMeta::from_derive(&ast);
    let mut databases = get_databases(&meta.table);
    let tables = get_tables();
    if meta.name.is_none() {
        if let Some(m) = tables.get(meta.returns.as_ref()) {
            meta.table.name = m.name.clone();
        }
    }
    let first = databases.remove(0);
    TokenStream::from(impl_Insert(first.as_ref(), &meta.table, &meta.ident, &meta.returns))
}

#[proc_macro_derive(FromRow, attributes(ormlite))]
pub fn expand_derive_fromrow(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let meta = TableMeta::from_derive(&ast);

    let databases = get_databases(&meta);
    let tables = get_tables();

    let expanded = databases.iter().map(|db| {
        let db = db.as_ref();
        let impl_FromRow = impl_FromRow(db, &meta, tables);
        let impl_from_row_using_aliases = impl_from_row_using_aliases(db, &meta, tables);
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
    let Data::Struct(data) = &ast.data else {
        panic!("Only structs can derive Model");
    };

    let table_meta = TableMeta::from_derive(&ast);
    let databases = get_databases(&table_meta);
    let impl_TableMeta = impl_TableMeta(&table_meta, table_meta.pkey.as_deref());
    TokenStream::from(impl_TableMeta)
}

#[proc_macro_derive(IntoArguments, attributes(ormlite))]
pub fn expand_derive_into_arguments(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let Data::Struct(data) = &ast.data else {
        panic!("Only structs can derive Model");
    };

    let meta = TableMeta::from_derive(&ast);
    let databases = get_databases(&meta);

    let expanded = databases.iter().map(|db| {
        let impl_IntoArguments = impl_IntoArguments(db.as_ref(), &meta);
        impl_IntoArguments
    });
    TokenStream::from(quote! {
        #(#expanded)*
    })
}

/// This is a no-op marker trait that allows the migration tool to know when a user has
/// manually implemented a type.
///
/// This is useful for having data that's a string in the database, but a strum::EnumString in code.
#[proc_macro_derive(ManualType)]
pub fn expand_derive_manual_type(input: TokenStream) -> TokenStream {
    TokenStream::new()
}
