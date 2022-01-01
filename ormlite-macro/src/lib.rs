#![allow(non_snake_case)]
use crate::attr::{
    ColumnAttributes, ColumnMeta, ColumnMetaBuilder, ModelAttributes, TableMeta, TableMetaBuilder,
};
use crate::codegen::common::OrmliteCodegen;
use proc_macro::TokenStream;

use crate::util::get_fields;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) mod attr;
pub(crate) mod codegen;
pub(crate) mod util;

fn finish_table_meta(ast: &DeriveInput, mut builder: TableMetaBuilder) -> TableMeta {
    let model = &ast.ident;
    let model_lowercased = model.to_string().to_lowercase();
    builder.table_name = builder.table_name.or(Some(model_lowercased.clone()));
    let fields = get_fields(&ast);

    let cols = fields
        .iter()
        .map(|f| build_column_meta(f))
        .collect::<Vec<ColumnMeta>>();
    let mut primary_key = cols.iter().filter(|c| c.marked_primary_key).next();
    if primary_key.is_none() {
        for f in cols.iter() {
            eprintln!("checking for primary key on {}", f.column_name);
            if [
                "id".to_string(),
                "uuid".to_string(),
                format!("{}_id", model_lowercased),
                format!("{}_uuid", model_lowercased),
            ]
            .contains(&f.column_name)
            {
                primary_key = Some(f);
                break;
            }
        }
    }
    if primary_key.is_none() {
        panic!("No column marked with #[ormlite(primary_key)], and no column named id, uuid, {0}_id, or {0}_uuid", model_lowercased);
    } else {
        builder.primary_key(primary_key.unwrap().column_name.clone());
    }
    builder.columns(cols);
    builder.build().unwrap()
}

fn partial_build_table_meta(ast: &DeriveInput) -> TableMetaBuilder {
    let mut builder = TableMetaBuilder::default();
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

fn build_column_meta(f: &syn::Field) -> ColumnMeta {
    let mut builder = ColumnMetaBuilder::default();
    builder.column_name(f.ident.as_ref().unwrap().to_string());
    builder.column_type(f.ty.clone());
    builder.marked_primary_key(false);
    for attr in f.attrs.iter().filter(|a| a.path.is_ident("ormlite")) {
        let args: ColumnAttributes = attr.parse_args().unwrap();
        if args.primary_key {
            builder.marked_primary_key(true);
        }
    }
    builder.build().unwrap()
}

#[proc_macro_derive(Model, attributes(ormlite))]
pub fn expand_ormlite_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let builder = partial_build_table_meta(&ast);
    let table_meta = finish_table_meta(&ast, builder);

    let impl_Model = codegen::DB::impl_Model(&ast, &table_meta);
    let impl_HasQueryBuilder = codegen::DB::impl_HasQueryBuilder(&ast, &table_meta);
    let impl_HasModelBuilder = codegen::DB::impl_HasModelBuilder(&ast, &table_meta);

    let struct_ModelBuilder = codegen::DB::struct_ModelBuilder(&ast, &table_meta);
    let impl_ModelBuilder = codegen::DB::impl_ModelBuilder(&ast, &table_meta);

    let expanded = quote! {
        #impl_Model
        #impl_HasModelBuilder
        #impl_HasQueryBuilder

        #struct_ModelBuilder
        #impl_ModelBuilder
    };
    TokenStream::from(expanded)
}
