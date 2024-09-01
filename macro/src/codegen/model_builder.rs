use crate::codegen::common::OrmliteCodegen;
use crate::codegen::insert::impl_ModelBuilder__insert;
use crate::codegen::update::impl_ModelBuilder__update;
use ormlite_attr::ModelMeta;
use ormlite_attr::TableMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn struct_ModelBuilder(ast: &DeriveInput, attr: &ModelMeta) -> TokenStream {
    let model = &attr.ident;
    let model_builder = attr.builder_struct();
    let vis = &ast.vis;

    let settable = attr.database_columns().map(|c| {
        let name = &c.ident;
        let ty = &c.ty;
        quote! { #name: std::option::Option<#ty> }
    });

    let methods = attr.database_columns().map(|c| {
        let name = &c.ident;
        let ty = &c.ty;
        if ty.is_string() {
            quote! {
                pub fn #name<T: Into<String>>(mut self, #name: T) -> Self {
                    self.#name = Some(#name.into());
                    self
                }
            }
        } else {
            quote! {
                pub fn #name(mut self, #name: #ty) -> Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });

    let fields_none = attr.database_columns().map(|c| {
        let name = &c.ident;
        quote! {
            #name: None
        }
    });

    quote! {
        #vis struct #model_builder<'a> {
            #(#settable,)*
            updating: Option<&'a #model>,
        }

        impl<'a> std::default::Default for #model_builder<'a> {
            fn default() -> Self {
                Self {
                    #(#fields_none,)*
                    updating: None,
                }
            }
        }

        impl<'a> #model_builder<'a> {
            #(#methods)*

        }
    }
}

pub fn impl_ModelBuilder__build(attr: &TableMeta) -> TokenStream {
    let unpack = attr.database_columns().map(|c| {
        let c = &c.ident;
        let msg = format!("Tried to build a model, but the field `{}` was not set.", c);
        quote! { let #c = self.#c.expect(#msg); }
    });

    let fields = attr.database_columns().map(|c| &c.ident);

    let skipped_fields = attr.columns.iter().filter(|&c| c.skip).map(|c| {
        let id = &c.ident;
        quote! {
            #id: Default::default()
        }
    });

    quote! {
        fn build(self) -> Self::Model {
            #( #unpack )*
            Self::Model {
                #( #fields, )*
                #( #skipped_fields, )*
            }
        }
    }
}

pub fn impl_ModelBuilder(db: &dyn OrmliteCodegen, attr: &ModelMeta) -> TokenStream {
    let partial_model = attr.builder_struct();
    let model = &attr.ident;

    let impl_ModelBuilder__insert = impl_ModelBuilder__insert(db, &attr.table);
    let impl_ModelBuilder__update = impl_ModelBuilder__update(db, attr);
    let impl_ModelBuilder__build = impl_ModelBuilder__build(&attr.table);

    let build_modified_fields = attr.database_columns().map(|c| {
        let name = &c.ident;
        let name_str = &c.name;
        quote! {
            if self.#name.is_some() {
                ret.push(#name_str);
            }
        }
    });

    let db = db.database_ts();
    quote! {
        impl<'a> ::ormlite::model::ModelBuilder<'a, #db> for #partial_model<'a> {
            type Model = #model;
            #impl_ModelBuilder__insert
            #impl_ModelBuilder__update
            #impl_ModelBuilder__build

            fn modified_fields(&self) -> Vec<&'static str> {
                let mut ret = Vec::new();
                #(#build_modified_fields)*
                ret
            }
        }
    }
}
