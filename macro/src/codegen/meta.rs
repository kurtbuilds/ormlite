use proc_macro2::TokenStream;
use quote::quote;
use ormlite_attr::TableMetadata;
use ormlite_attr::ModelMetadata;

pub fn impl_TableMeta(meta: &TableMetadata, pkey: Option<&str>) -> TokenStream {
    let model = &meta.struct_name;
    let table_name = &meta.table_name;
    let id = match pkey {
        Some(id) => quote! { Some(#id) },
        None => quote! { None },
    };

    let field_names = meta.database_columns()
        .map(|c| c.column_name.to_string());

    quote! {
        impl ::ormlite::model::TableMeta for #model {
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

pub fn impl_JoinMeta(attr: &ModelMetadata) -> TokenStream {
    let model = &attr.inner.struct_name;
    let id_type = &attr.pkey.column_type;
    let id = &attr.pkey.identifier;

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
