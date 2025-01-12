use crate::MetadataCache;
use ormlite_attr::TableMeta;
use proc_macro2::TokenStream;
use quote::quote;

pub fn static_join_descriptions(attr: &TableMeta, metadata_cache: &MetadataCache) -> TokenStream {
    let joins = attr.columns.iter().filter(|c| c.is_join()).map(|c| {
        let join = c.join.as_ref().expect("not a join");
        let field = &c.ident.to_string();
        let column = &c.name;
        let struct_name = c.joined_struct_name().unwrap();
        let joined_table = metadata_cache
            .get(&struct_name)
            .expect(&format!("Did not find metadata for joined struct: {}", struct_name));
        let foreign_table = &joined_table.table.name;
        let foreign_key = &joined_table.pkey.name;

        let columns = joined_table.database_columns().map(|c| &c.name);
        let body = match join {
            ormlite_attr::Join::ManyToOne { column } => {
                quote! {
                    ::ormlite::__private::JoinDescription::ManyToOne {
                        columns: &[
                            #(
                                #columns,
                            )*
                        ],
                        foreign_table: #foreign_table,
                        local_column: #column,
                        field: #field,
                        foreign_key: #foreign_key,
                    }
                }
            }
            ormlite_attr::Join::ManyToMany { table } => todo!(),
            ormlite_attr::Join::OneToMany { model, field } => todo!(),
        };
        let ident = &c.ident;
        quote! {
            pub fn #ident() -> ::ormlite::__private::JoinDescription {
                #body
            }
        }
    });

    let model = &attr.ident;
    quote! {
        impl #model {
            #(
                #joins
            )*
        }
    }
}
