use ormlite_attr::Ident;
use ormlite_attr::ModelMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn struct_InsertModel(ast: &DeriveInput, attr: &ModelMeta) -> TokenStream {
    let Some(insert_model) = &attr.insert_struct else {
        return quote! {};
    };
    let vis = &ast.vis;
    let struct_fields = attr.columns.iter().filter(|c| !c.is_default()).map(|c| {
        let id = &c.ident;
        let ty = &c.ty;
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
