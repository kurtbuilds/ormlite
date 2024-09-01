use crate::MetadataCache;
use itertools::Itertools;
use ormlite_attr::ColumnMeta;
use ormlite_attr::Ident;
use ormlite_attr::ModelMeta;
use ormlite_attr::TableMeta;
use ormlite_attr::{InnerType, Type};
use ormlite_core::query_builder::Placeholder;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Cow;

pub fn generate_conditional_bind(c: &ColumnMeta) -> TokenStream {
    let name = &c.ident;
    if c.is_join() {
        quote! {
            if let Some(value) = self.#name {
                q = q.bind(value._id());
            }
        }
    } else if c.json {
        quote! {
            if let Some(value) = self.#name {
                q = q.bind(::ormlite::types::Json(value));
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

fn recursive_primitive_types_ty<'a>(ty: &'a Type, cache: &'a MetadataCache) -> Vec<Cow<'a, InnerType>> {
    match ty {
        Type::Option(ty) => recursive_primitive_types_ty(ty, cache),
        Type::Vec(ty) => {
            let inner = recursive_primitive_types_ty(ty, cache);
            let inner = inner.into_iter().next().expect("Vec must have inner type");
            let inner: InnerType = inner.into_owned();
            vec![Cow::Owned(InnerType {
                path: vec![],
                ident: Ident::from("Vec"),
                args: Some(Box::new(inner)),
            })]
        }
        Type::Inner(p) => vec![Cow::Borrowed(p)],
        Type::Join(j) => {
            let joined = cache.get(&j.inner_type_name()).expect("Join type not found");
            recursive_primitive_types(joined, cache)
        }
    }
}

fn recursive_primitive_types<'a>(table: &'a ModelMeta, cache: &'a MetadataCache) -> Vec<Cow<'a, InnerType>> {
    table
        .columns
        .iter()
        .map(|c| recursive_primitive_types_ty(&c.ty, cache))
        .flatten()
        .collect()
}

pub(crate) fn table_primitive_types<'a>(attr: &'a TableMeta, cache: &'a MetadataCache) -> Vec<Cow<'a, InnerType>> {
    attr.columns
        .iter()
        .filter(|c| !c.skip)
        .filter(|c| !c.json)
        .map(|c| recursive_primitive_types_ty(&c.ty, cache))
        .flatten()
        .unique()
        .collect()
}

pub fn from_row_bounds<'a>(
    db: &dyn OrmliteCodegen,
    attr: &'a TableMeta,
    cache: &'a MetadataCache,
) -> impl Iterator<Item = TokenStream> + 'a {
    let database = db.database_ts();
    table_primitive_types(attr, cache).into_iter().map(move |ty| {
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
pub fn insertion_binding(c: &ColumnMeta) -> TokenStream {
    let name = &c.ident;
    if c.is_join() {
        quote! {
            q = q.bind(#name._id());
        }
    } else if c.json {
        quote! {
            q = q.bind(::ormlite::types::Json(model.#name));
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
