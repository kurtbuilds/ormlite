use proc_macro2::TokenStream;
use quote::quote;
use ormlite_attr::{ColumnMetadata, Ident, TableMetadata};
use crate::codegen::common::from_row_bounds;
use crate::MetadataCache;

pub fn impl_FromRow(attr: &TableMetadata, cache: &MetadataCache) -> TokenStream {
    let bounds = from_row_bounds(attr, cache);

    let prefix_branches = attr.columns.iter().filter(|c| c.is_join()).map(|c| {
        let name = &c.identifier.to_string();
        let iden = &c.identifier;
        let meta = cache.get(c.joined_struct_name().unwrap().as_str())
            .expect("Joined struct not found");
        let result = if c.is_join_many() {
            unimplemented!("Join<Vec<...>> isn't supported quite yet...");
        } else {
            let prefixed_columns = meta.columns.iter().map(|c| {
                format!("__{}__{}", iden, c.identifier)
            });
            let path = c.joined_model();
            quote! {
                #path::from_row_using_aliases(row, &[
                    #(
                        #prefixed_columns,
                    )*
                ])?
            }
        };
        quote! {
            #name => {
                model.#iden = ::ormlite::model::Join::_query_result(#result);
            }
        }
    });

    let field_names = attr.database_columns()
        .map(|c| &c.column_name);

    let map_join = if attr.columns.iter().any(|c| c.is_join()) {
        quote! {
            let mut prefixes = row.columns().iter().filter_map(|c| {
                let name = ::ormlite::Column::name(c);
                if name.starts_with("__") {
                    name.rsplitn(2, "__").last().map(|s| &s[2..])
                } else {
                    None
                }
            })
                .collect::<Vec<_>>();
            prefixes.sort();
            prefixes.dedup();
            for prefix in prefixes {
                match prefix {
                    #(
                        #prefix_branches
                    )*
                    _ => {
                        return Err(::ormlite::SqlxError::Decode(
                            Box::new(::ormlite::Error::OrmliteError(format!("Unknown column prefix: {}", prefix))),
                        ));
                    }
                }
            }
        }
    } else {
        TokenStream::new()
    };
    let model = &attr.struct_name;
    quote! {
        impl<'a, R: ::ormlite::Row> ::ormlite::model::FromRow<'a, R> for #model
            where
                &'a str: ::ormlite::ColumnIndex<R>,
                #(
                    #bounds
                )*
        {
            fn from_row(row: &'a R) -> ::std::result::Result<Self, ::ormlite::SqlxError> {
                let mut model = Self::from_row_using_aliases(row, &[
                    #(
                        #field_names,
                    )*
                ])?;
                #map_join
                Ok(model)
            }
        }
    }
}


pub fn impl_from_row_using_aliases(attr: &TableMetadata, metadata_cache: &MetadataCache) -> TokenStream {
    let fields = attr.all_fields();
    let bounds = from_row_bounds(attr, &metadata_cache);
    let mut incrementer = 0usize..;
    let columns = attr.columns.iter()
        .map(|c| {
            let index = incrementer.next().unwrap();
            let get = quote! { aliases[#index] };
            from_row_for_column(get, c)
        })
        .collect::<Vec<_>>();

    let model = &attr.struct_name;
    quote! {
        impl #model {
            pub fn from_row_using_aliases<'a, R: ::ormlite::Row>(row: &'a R, aliases: &'a [&str]) -> ::std::result::Result<Self, ::ormlite::SqlxError>
                where
                    &'a str: ::ormlite::ColumnIndex<R>,
                    #(
                        #bounds
                    )*
            {
                #(
                    #columns
                )*
                Ok(Self { #(#fields,)* })
            }
        }
    }
}


/// `name` renames the column. Can pass `col.column_name` if it's not renamed.
pub fn from_row_for_column(get_value: TokenStream, col: &ColumnMetadata) -> TokenStream {
    let id = &col.identifier;
    let ty = &col.column_type;
    if col.skip {
        quote! {
            let #id = Default::default();
        }
    } else if col.is_join() {
        let id_id = Ident::new(&format!("{}_id", id));
        quote! {
            let #id_id: <#ty as ::ormlite::model::JoinMeta>::IdType = row.try_get(#get_value)?;
            let #id = ::ormlite::model::Join::new_with_id(#id_id);
        }
    } else {
        quote! {
            let #id: #ty = row.try_get(#get_value)?;
        }
    }
}
