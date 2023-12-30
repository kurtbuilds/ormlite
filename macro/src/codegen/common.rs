use std::borrow::Cow;
use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use crate::MetadataCache;
use itertools::Itertools;
use ormlite_attr::Ident;
use ormlite_attr::ColumnMetadata;
use ormlite_attr::ModelMetadata;
use ormlite_attr::TableMetadata;
use ormlite_attr::{InnerType, TType};


pub fn generate_conditional_bind(c: &ColumnMetadata) -> TokenStream {
    let name = &c.identifier;
    if c.is_join() {
        quote! {
            if let Some(value) = self.#name {
                q = q.bind(value._id());
            }
        }
    } else {
        quote! {
            if let Some(value) = self.#name {
                q = q.bind(value);
            }
        }
    }
}

/// bool whether the given type is `String`
fn ty_is_string(ty: &syn::Type) -> bool {
    let p = match ty {
        syn::Type::Path(p) => p,
        _ => return false,
    };
    p.path.segments.last().map(|s| s.ident == "String").unwrap_or(false)
}



fn recursive_primitive_types_ty<'a>(ty: &'a TType, cache: &'a MetadataCache) -> Vec<Cow<'a, InnerType>> {
    match ty {
        TType::Option(ty) => {
            recursive_primitive_types_ty(ty, cache)
        }
        TType::Vec(ty) => {
            let inner = recursive_primitive_types_ty(ty, cache);
            let inner = inner.into_iter().next().expect("Vec must have inner type");
            let inner: InnerType = inner.into_owned();
            vec![Cow::Owned(InnerType {
                path: vec![],
                ident: Ident("Vec".to_string()),
                args: Some(Box::new(inner)),
            })]
        }
        TType::Inner(p) => vec![Cow::Borrowed(p)],
        TType::Join(j) => {
            let joined = cache.get(&j.inner_type_name()).expect("Join type not found");
            recursive_primitive_types(joined, cache)
        }
    }
}


fn recursive_primitive_types<'a>(table: &'a ModelMetadata, cache: &'a MetadataCache) -> Vec<Cow<'a, InnerType>> {
    table.inner.columns.iter()
        .map(|c| {
            recursive_primitive_types_ty(&c.column_type, cache)
        })
        .flatten()
        .collect()
}

pub(crate) fn table_primitive_types<'a>(attr: &'a TableMetadata, cache: &'a MetadataCache) -> Vec<Cow<'a, InnerType>> {
    attr.columns.iter()
        .filter(|c| !c.skip)
        .filter(|c| !c.experimental_encode_as_json)
        .map(|c| recursive_primitive_types_ty(&c.column_type, cache))
        .flatten()
        .unique()
        .collect()
}

pub fn from_row_bounds<'a>(db: &dyn OrmliteCodegen, attr: &'a TableMetadata, cache: &'a MetadataCache) -> impl Iterator<Item=proc_macro2::TokenStream> + 'a {
    let database = db.database_ts();
    table_primitive_types(attr, cache)
        .into_iter()
        .map(move |ty| {
            quote! {
                #ty: ::ormlite::decode::Decode<'a, #database>,
                #ty: ::ormlite::types::Type<#database>,
            }
        })
}

fn is_vec(p: &syn::Path) -> bool {
    let Some(segment) = p.segments.last() else {
        return false;
    };
    segment.ident == "Vec"
}

/// Used to bind fields to the query upon insertion, update, etc.
/// Assumed Bindings:
/// - `model`: model struct
/// - `q`: sqlx query
pub fn insertion_binding(c: &ColumnMetadata) -> TokenStream {
    let name = &c.identifier;
    if c.is_join() {
        quote! {
            q = q.bind(#name._id());
        }
    } else {
        quote! {
            q = q.bind(model.#name);
        }
    }
}

pub trait OrmliteCodegen {
    fn database_ts(&self) -> TokenStream;
    fn placeholder_ts(&self) -> TokenStream;
    // A placeholder that works at the phase when its invoked (e.g. during comp time, it can be used.
    // Compare to placeholder_ts, which is just the tokens of a placeholder, and therefore can't be "used" until runtime.
    fn placeholder(&self) -> Placeholder;
    fn row(&self) -> TokenStream;

}
