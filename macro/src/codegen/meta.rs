use ormlite_attr::ModelMeta;
use ormlite_attr::TableMeta;
use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_TableMeta(table: &TableMeta, pkey: Option<&str>) -> TokenStream {
    let ident = &table.ident;
    let table_name = &table.name;
    let id = match pkey {
        Some(id) => quote! { Some(#id) },
        None => quote! { None },
    };

    let field_names = table.database_columns().map(|c| c.name.to_string());

    quote! {
        impl ::ormlite::model::TableMeta for #ident {
            fn table_name() -> &'static str {
                #table_name
            }

            fn table_columns() -> &'static [&'static str] {
                &[#(#field_names,)*]
            }

            fn primary_key() -> Option<&'static str> {
                #id
            }
        }
    }
}

pub fn impl_JoinMeta(attr: &ModelMeta) -> TokenStream {
    let model = &attr.ident;
    let id_type = &attr.pkey.ty;
    let id = &attr.pkey.ident;

    quote! {
        impl ::ormlite::model::JoinMeta for #model {
            type IdType = #id_type;
            fn _id(&self) -> Self::IdType {
                // clone is identical to Copy for most id types, but lets us use cloneable types like String.
                self.#id.clone()
            }
        }
    }
}
