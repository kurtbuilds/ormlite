use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Data::Struct;
use syn::{DataStruct, DeriveInput, Field, Fields, FieldsNamed};

pub fn box_future() -> TokenStream {
    quote!(::ormlite::BoxFuture)
}


pub trait DeriveInputExt {
    fn fields(&self) -> syn::punctuated::Iter<Field>;
}

impl DeriveInputExt for DeriveInput {
    fn fields(&self) -> syn::punctuated::Iter<Field> {
        let fields = match &self.data {
            Struct(DataStruct { ref fields, .. }) => fields,
            _ => panic!("#[ormlite] can only be used on structs"),
        };
        let fields = match fields {
            Fields::Named(FieldsNamed { named, .. }) => named,
            _ => panic!("#[ormlite] can only be used on structs with named fields"),
        };
        fields.iter()
    }
}

pub trait FieldExt {
    fn name(&self) -> String;
}

impl FieldExt for Field {
    fn name(&self) -> String {
        self.ident.as_ref().unwrap().to_string().replace("r#", "")
    }
}