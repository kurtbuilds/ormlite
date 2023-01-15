#![allow(unused)]
#![allow(non_snake_case)]

use ormlite_attr::{ColumnAttributes, ColumnMetadata, TableMetadata, TableMetadataBuilder, ColumnMetadataBuilder, ModelAttributes, SyndecodeError, load_from_project, LoadOptions};
use ormlite_core::config::get_var_model_folders;
use crate::codegen::common::OrmliteCodegen;
use proc_macro::TokenStream;
use std::borrow::Borrow;

use quote::quote;
use lazy_static::lazy_static;
use syn::{Data, DeriveInput, Item, ItemStruct, parse_macro_input};
use ormlite_attr::DeriveInputExt;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::ops::Deref;

mod codegen;
mod util;

pub(crate) type MetadataCache = HashMap<String, TableMetadata>;

thread_local! {
    static TABLES: RwLock<MetadataCache> = RwLock::new(HashMap::new());
}

fn load_project_metadata(cache: &RwLock<MetadataCache>) {
    let paths = get_var_model_folders();
    let paths = paths.iter().map(|p| p.as_path()).collect::<Vec<_>>();
    if !cache.read().unwrap().is_empty() {
        return;
    }
    let mut lock = cache.write().unwrap();
    let vec_meta = load_from_project(&paths, &LoadOptions::default()).expect("Failed to preload models.");
    for meta in vec_meta {
        let name = meta.struct_name.to_string();
        lock.insert(name, meta);
    }
}

/// Derive macro for `#[derive(Model)]` It additionally generates FromRow for the struct, since
/// Model requires FromRow.
#[proc_macro_derive(Model, attributes(ormlite))]
pub fn expand_ormlite_model(input: TokenStream) -> TokenStream {
    TABLES.with(load_project_metadata);

    let input2 = input.clone();
    let ast = parse_macro_input!(input2 as DeriveInput);
    let Data::Struct(data) = &ast.data else { panic!("Only structs can derive Model"); };

    let table_meta = TableMetadata::try_from(&ast).unwrap();
    if table_meta.primary_key.is_none() {
        panic!("No column marked with #[ormlite(primary_key)], and no column named id, uuid, {0}_id, or {0}_uuid", table_meta.table_name);
    }

    let impl_Model = TABLES.with(|t| {
        let read = t.read().unwrap();
        codegen::DB::impl_Model(&ast, &table_meta, &read)
    });
    let impl_FromRow = TABLES.with(|t| {
        let read = t.read().unwrap();
        codegen::DB::impl_FromRow(&ast, &table_meta, &read)
    });
    let impl_from_row_using_aliases = TABLES.with(|t| {
        let read = t.read().unwrap();
        codegen::DB::impl_from_row_using_aliases(&ast, &table_meta, &read)
    });

    let struct_ModelBuilder = codegen::DB::struct_ModelBuilder(&ast, &table_meta);
    let impl_ModelBuilder = codegen::DB::impl_ModelBuilder(&ast, &table_meta);

    let struct_InsertModel = codegen::DB::struct_InsertModel(&ast, &table_meta);
    let impl_InsertModel = codegen::DB::impl_InsertModel(&ast, &table_meta);

    let expanded = quote! {
        #impl_Model
        #impl_FromRow
        #impl_from_row_using_aliases

        #struct_ModelBuilder
        #impl_ModelBuilder

        #struct_InsertModel
        #impl_InsertModel
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(FromRow)]
pub fn expand_derive_fromrow(input: TokenStream) -> TokenStream {
    TABLES.with(load_project_metadata);

    let input2 = input.clone();
    let ast = parse_macro_input!(input2 as DeriveInput);
    let Data::Struct(data) = &ast.data else { panic!("Only structs can derive Model"); };

    let table_meta = TableMetadata::try_from(&ast).unwrap();

    let impl_FromRow = TABLES.with(|t| {
        let read = t.read().unwrap();
        codegen::DB::impl_FromRow(&ast, &table_meta, &read)
    });
    let impl_from_row_using_aliases = TABLES.with(|t| {
        let read = t.read().unwrap();
        codegen::DB::impl_from_row_using_aliases(&ast, &table_meta, &read)
    });

    let expanded = quote! {
        #impl_FromRow
        #impl_from_row_using_aliases
    };

    TokenStream::from(expanded)
}