use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Data::Struct;
use syn::{DataStruct, DeriveInput, Field, Fields, FieldsNamed};

pub fn box_future() -> TokenStream {
    quote!(::ormlite::BoxFuture)
}

/// Given derive input of a struct, get the fields of the struct.
pub fn get_fields(ast: &DeriveInput) -> &Punctuated<Field, Comma> {
    let fields = match &ast.data {
        Struct(DataStruct { ref fields, .. }) => fields,
        _ => panic!("#[ormlite] can only be used on structs"),
    };
    let fields = match fields {
        Fields::Named(FieldsNamed { named, .. }) => named,
        _ => panic!("#[ormlite] can only be used on structs with named fields"),
    };
    fields
}
