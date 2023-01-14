#![allow(unused)]
#![allow(non_snake_case)]

use ormlite_attr::{ColumnAttributes, ColumnMetadata, TableMetadata, TableMetadataBuilder, ColumnMetadataBuilder, ModelAttributes, SyndecodeError};
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

pub(crate) type MetadataLookup = HashMap<String, TableMetadata>;

thread_local! {
    static TABLES: RwLock<MetadataLookup> = RwLock::new(HashMap::new());
}

fn manually_read_tables() -> Vec<(String, TableMetadata)> {
    vec![
        ("Person".to_string(), TableMetadata {
            table_name: "person".to_string(),
            struct_name: syn::Ident::new("Person", proc_macro2::Span::call_site()),
            primary_key: Some("id".to_string()),
            insert_struct: None,
            columns: vec![
                ColumnMetadata {
                    column_name: "id".to_string(),
                    column_type: syn::parse_str("Uuid").unwrap(),
                    marked_primary_key: false,
                    has_database_default: false,
                    identifier: syn::Ident::new("id", proc_macro2::Span::call_site()),
                    many_to_one_key: None,
                    many_to_many_table_name: None,
                    one_to_many_foreign_key: None,
                },
                ColumnMetadata {
                    column_name: "name".to_string(),
                    column_type: syn::parse_str("String").unwrap(),
                    marked_primary_key: false,
                    has_database_default: false,
                    identifier: syn::Ident::new("name", proc_macro2::Span::call_site()),
                    many_to_one_key: None,
                    many_to_many_table_name: None,
                    one_to_many_foreign_key: None,
                },
                ColumnMetadata {
                    column_name: "age".to_string(),
                    column_type: syn::parse_str("u8").unwrap(),
                    marked_primary_key: false,
                    has_database_default: false,
                    identifier: syn::Ident::new("age", proc_macro2::Span::call_site()),
                    many_to_one_key: None,
                    many_to_many_table_name: None,
                    one_to_many_foreign_key: None,
                },
                ColumnMetadata {
                    column_name: "org_id".to_string(),
                    column_type: syn::parse_str("Uuid").unwrap(),
                    marked_primary_key: false,
                    has_database_default: false,
                    identifier: syn::Ident::new("org_id", proc_macro2::Span::call_site()),
                    many_to_one_key: None,
                    many_to_many_table_name: None,
                    one_to_many_foreign_key: None,
                },
                ColumnMetadata {
                    column_name: "organization".to_string(),
                    column_type: syn::parse_str("Join<Organization>").unwrap(),
                    marked_primary_key: false,
                    has_database_default: false,
                    identifier: syn::Ident::new("organization", proc_macro2::Span::call_site()),
                    many_to_one_key: syn::Ident::new("org_id", proc_macro2::Span::call_site()).into(),
                    many_to_many_table_name: None,
                    one_to_many_foreign_key: None,
                },
            ],
        }),
        ("Organization".to_string(), TableMetadata {
            table_name: "organization".to_string(),
            primary_key: Some("id".to_string()),
            struct_name: syn::Ident::new("Organization", proc_macro2::Span::call_site()),
            insert_struct: None,
            columns: vec![
                ColumnMetadata {
                    column_name: "id".to_string(),
                    column_type: syn::parse_str("Uuid").unwrap(),
                    marked_primary_key: false,
                    has_database_default: false,
                    identifier: syn::Ident::new("id", proc_macro2::Span::call_site()),
                    many_to_one_key: None,
                    many_to_many_table_name: None,
                    one_to_many_foreign_key: None,
                },
                ColumnMetadata {
                    column_name: "name".to_string(),
                    column_type: syn::parse_str("String").unwrap(),
                    marked_primary_key: false,
                    has_database_default: false,
                    identifier: syn::Ident::new("name", proc_macro2::Span::call_site()),
                    many_to_one_key: None,
                    many_to_many_table_name: None,
                    one_to_many_foreign_key: None,
                },
            ],
        })
    ]
}

/// Derive macro for `#[derive(Model)]` It additionally generates FromRow for the struct, since
/// Model requires FromRow.
#[proc_macro_derive(Model, attributes(ormlite))]
pub fn expand_ormlite_model(input: TokenStream) -> TokenStream {
    TABLES.with(|t| {
        if !t.read().unwrap().is_empty() {
            return;
        }
        let mut lock = t.write().unwrap();
        let meta = manually_read_tables();
        for (s, m) in meta {
            lock.insert(s, m);
        }
    });

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
    let impl_FromRow = codegen::DB::impl_FromRow(&ast, &table_meta);

    let struct_ModelBuilder = codegen::DB::struct_ModelBuilder(&ast, &table_meta);
    let impl_ModelBuilder = codegen::DB::impl_ModelBuilder(&ast, &table_meta);

    let struct_InsertModel = codegen::DB::struct_InsertModel(&ast, &table_meta);
    let impl_InsertModel = codegen::DB::impl_InsertModel(&ast, &table_meta);

    let expanded = quote! {
        #impl_Model
        #impl_FromRow

        #struct_ModelBuilder
        #impl_ModelBuilder

        #struct_InsertModel
        #impl_InsertModel
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(FromRow)]
pub fn expand_derive_fromrow(input: TokenStream) -> TokenStream {
    let input2 = input.clone();
    let ast = parse_macro_input!(input2 as DeriveInput);
    let Data::Struct(data) = &ast.data else { panic!("Only structs can derive Model"); };

    let table_meta = TableMetadata::try_from(&ast).unwrap();

    let impl_FromRow = codegen::DB::impl_FromRow(&ast, &table_meta);

    let expanded = quote! {
        #impl_FromRow
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn index(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}