#![allow(non_snake_case)]
use crate::attr::{parse_attrs, Column, TableAttr, TableAttrBuilder};
use crate::codegen::common::OrmliteCodegen;
use proc_macro::TokenStream;

use crate::util::get_fields;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) mod attr;
pub(crate) mod codegen;
pub(crate) mod util;

fn finish_attr_builder(ast: &DeriveInput, mut attr_builder: TableAttrBuilder) -> TableAttr {
    let model = &ast.ident;
    let model_lowercased = model.to_string().to_lowercase();

    attr_builder.table_name = attr_builder.table_name.or(Some(model_lowercased.clone()));

    let fields = get_fields(&ast);
    attr_builder.columns(
        fields
            .iter()
            .map(|f| Column {
                column_name: f.ident.as_ref().unwrap().to_string(),
                column_type: f.ty.clone(),
            })
            .collect(),
    );

    attr_builder.primary_key_column = attr_builder.primary_key_column.or_else(|| {
        fields
            .iter()
            .filter_map(|f| {
                if [
                    "id".to_string(),
                    "uuid".to_string(),
                    format!("{}_id", model_lowercased),
                    format!("{}_uuid", model_lowercased),
                ]
                .contains(&f.ident.as_ref().unwrap().to_string())
                {
                    Some(f.ident.as_ref().unwrap().to_string())
                } else {
                    None
                }
            })
            .next()
    });

    attr_builder.build().unwrap()
}

#[proc_macro_derive(Model, attributes(ormlite))]
pub fn expand_ormlite_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let attr_builder = parse_attrs(&ast.attrs, ast.ident.span()).unwrap();
    let attr = finish_attr_builder(&ast, attr_builder);

    let impl_Model = codegen::DB::impl_Model(&ast, &attr);
    let impl_BuildsQueryBuilder = codegen::DB::impl_BuildsQueryBuilder(&ast, &attr);
    let impl_BuildsPartialModel = codegen::DB::impl_BuildsPartialModel(&ast, &attr);

    let struct_PartialModel = codegen::DB::struct_PartialModel(&ast, &attr);
    let impl_PartialModel = codegen::DB::impl_PartialModel(&ast, &attr);

    let expanded = quote! {
        #impl_Model
        #impl_BuildsPartialModel
        #impl_BuildsQueryBuilder

        #struct_PartialModel
        #impl_PartialModel
    };
    TokenStream::from(expanded)
}
