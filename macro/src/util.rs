use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Data::Struct;
use syn::{DataStruct, DeriveInput, Field, Fields, FieldsNamed};

pub fn box_fut_ts() -> TokenStream {
    quote!(::ormlite::BoxFuture)
}
