use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use ormlite_attr::{Ident, ModelMetadata, TableMetadata};

pub fn struct_InsertModel(ast: &DeriveInput, attr: &ModelMetadata) -> TokenStream {
    let Some(insert_model) = &attr.insert_struct else {
        return quote! {};
    };
    let insert_model = Ident::new(insert_model);
    let vis = &ast.vis;
    let struct_fields = attr.inner.columns.iter()
        .filter(|c| !c.is_default())
        .map(|c| {
            let id = &c.identifier;
            let ty = &c.column_type;
            quote! {
                pub #id: #ty
            }
        });
    quote! {
        #[derive(Debug)]
        #vis struct #insert_model {
            #(#struct_fields,)*
        }
    }
}