use proc_macro2::TokenStream;
use quote::quote;
use ormlite_attr::TableMetadata;
use crate::codegen::common::OrmliteCodegen;

pub fn impl_TableMeta(meta: &TableMetadata) -> TokenStream {
    let model = &meta.struct_name;
    let table_name = &meta.table_name;
    let id = &meta.pkey.column_name;

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
                Some(#id)
            }
        }
    }
}

pub fn impl_JoinMeta(attr: &TableMetadata) -> TokenStream {
    let model = &attr.struct_name;
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
