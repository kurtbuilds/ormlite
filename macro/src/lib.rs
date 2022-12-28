#![allow(unused)]
#![allow(non_snake_case)]
use crate::attr::{
    ColumnAttributes, ModelAttributes,
};
use crate::codegen::common::OrmliteCodegen;
use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};
use metadata::{ColumnMetadata, TableMetadata};
use crate::metadata::{ColumnMetadataBuilder, TableMetadataBuilder};
use crate::util::DeriveInputExt;

pub(crate) mod attr;
pub(crate) mod codegen;
pub(crate) mod util;
mod metadata;

fn finish_table_meta(ast: &DeriveInput, mut builder: TableMetadataBuilder) -> TableMetadata {
    let model = &ast.ident;
    let model_lowercased = model.to_string().to_lowercase();
    builder.table_name = builder.table_name.or(Some(model_lowercased.clone()));

    let mut cols = ast.fields()
        .map(|f| build_column_meta(f))
        .collect::<Vec<ColumnMetadata>>();
    let mut primary_key = cols
        .iter()
        .filter(|c| c.marked_primary_key)
        .map(|m| m.column_name.clone())
        .next();
    if primary_key.is_none() {
        for f in cols.iter_mut() {
            if [
                "id".to_string(),
                "uuid".to_string(),
                format!("{}_id", model_lowercased),
                format!("{}_uuid", model_lowercased),
            ]
            .contains(&f.column_name)
            {
                primary_key = Some(f.column_name.clone());
                f.has_database_default = true;
                break;
            }
        }
    }
    if primary_key.is_none() {
        panic!("No column marked with #[ormlite(primary_key)], and no column named id, uuid, {0}_id, or {0}_uuid", model_lowercased);
    } else {
        builder.primary_key(primary_key.unwrap());
    }
    builder.columns(cols);
    builder.build().unwrap()
}

fn partial_build_table_meta(ast: &DeriveInput) -> TableMetadataBuilder {
    let mut builder = TableMetadata::builder();
    builder.insert_struct(None);
    for attr in ast.attrs.iter().filter(|a| a.path.is_ident("ormlite")) {
        let args: ModelAttributes = attr.parse_args().unwrap();
        if let Some(value) = args.table {
            builder.table_name(value.value());
        }
        if let Some(value) = args.insert {
            builder.insert_struct(Some(value.to_string()));
        }
    }
    builder
}

fn build_column_meta(f: &syn::Field) -> ColumnMetadata {
    let mut builder = ColumnMetadataBuilder::default();
    builder.column_name(f.ident.as_ref().unwrap().to_string());
    builder.column_type(f.ty.clone());
    builder.marked_primary_key(false);
    builder.has_database_default(false);
    for attr in f.attrs.iter().filter(|a| a.path.is_ident("ormlite")) {
        let args: ColumnAttributes = attr.parse_args().unwrap();
        if args.primary_key {
            builder.marked_primary_key(true);
            builder.has_database_default(true);
        }
        if args.default {
            builder.has_database_default(true);
        }
    }
    builder.build().unwrap()
}

#[proc_macro_derive(Model, attributes(ormlite))]
pub fn expand_ormlite_model(input: TokenStream) -> TokenStream {
    let input2 = input.clone();
    let ast = parse_macro_input!(input2 as DeriveInput);

    let builder = partial_build_table_meta(&ast);
    let table_meta = finish_table_meta(&ast, builder);

    let impl_Model = codegen::DB::impl_Model(&ast, &table_meta);
    let impl_HasModelBuilder = codegen::DB::impl_HasModelBuilder(&ast, &table_meta);

    let struct_ModelBuilder = codegen::DB::struct_ModelBuilder(&ast, &table_meta);
    let impl_ModelBuilder = codegen::DB::impl_ModelBuilder(&ast, &table_meta);

    let struct_InsertModel = codegen::DB::struct_InsertModel(&ast, &table_meta);
    let impl_InsertModel = codegen::DB::impl_InsertModel(&ast, &table_meta);

    let expanded = quote! {
        #impl_Model
        #impl_HasModelBuilder

        #struct_ModelBuilder
        #impl_ModelBuilder

        #struct_InsertModel
        #impl_InsertModel
    };

    TokenStream::from(expanded)
}
