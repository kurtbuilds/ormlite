use crate::MetadataCache;
use itertools::Itertools;
use ormlite_attr::{
    ColumnMetadata, DeriveInputExt, FieldExt, Ident, InnerType, TType, TableMetadata,
};
use ormlite_core::query_builder::Placeholder;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use sqlx::Column;
use std::borrow::Cow;
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Comma, Token};
use syn::{DeriveInput, Field};

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
    p.path
        .segments
        .last()
        .map(|s| s.ident == "String")
        .unwrap_or(false)
}

fn recursive_primitive_types_ty<'a>(
    ty: &'a TType,
    cache: &'a MetadataCache,
) -> Vec<Cow<'a, InnerType>> {
    match ty {
        TType::Option(ty) => recursive_primitive_types_ty(ty, cache),
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
            let joined = cache
                .get(&j.inner_type_name())
                .expect("Join type not found");
            recursive_primitive_types(joined, cache)
        }
    }
}

fn recursive_primitive_types<'a>(
    table: &'a TableMetadata,
    cache: &'a MetadataCache,
) -> Vec<Cow<'a, InnerType>> {
    table
        .columns
        .iter()
        .map(|c| recursive_primitive_types_ty(&c.column_type, cache))
        .flatten()
        .collect()
}

fn table_primitive_types<'a>(
    attr: &'a TableMetadata,
    cache: &'a MetadataCache,
) -> Vec<Cow<'a, InnerType>> {
    attr.columns
        .iter()
        .filter(|c| !c.skip)
        .map(|c| recursive_primitive_types_ty(&c.column_type, cache))
        .flatten()
        .unique()
        .collect()
}

pub fn from_row_bounds<'a>(
    attr: &'a TableMetadata,
    cache: &'a MetadataCache,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    table_primitive_types(attr, cache).into_iter().map(|ty| {
        quote! {
            #ty: ::ormlite::decode::Decode<'a, R::Database>,
            #ty: ::ormlite::types::Type<R::Database>,
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
}

#[cfg(test)]
mod test {
    use super::*;
    use ormlite_attr::*;

    #[test]
    fn test_all_bounds() {
        let mut cache = MetadataCache::new();
        let table = TableMetadata::new(
            "user",
            vec![
                ColumnMetadata::new("id", "u32"),
                ColumnMetadata::new("name", "String"),
                ColumnMetadata::new("organization_id", "u32"),
                ColumnMetadata::new_join("organization", "Organization"),
            ],
        );
        cache.insert("User".to_string(), table.clone());
        let table = TableMetadata::new(
            "organization",
            vec![
                ColumnMetadata::new("id", "u32"),
                ColumnMetadata::new("name", "String"),
                ColumnMetadata::new("is_active", "bool"),
            ],
        );
        cache.insert("Organization".to_string(), table.clone());

        let types_for_bound = table_primitive_types(&table, &cache);
        let types_for_bound = types_for_bound
            .into_iter()
            .map(|c| c.into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            types_for_bound,
            vec![
                InnerType::new("u32"),
                InnerType::new("String"),
                InnerType::new("bool"),
            ]
        );
        let bounds = from_row_bounds(&table, &cache);
        let bounds = quote! {
            #(#bounds)*
        };
        assert_eq!(
            bounds.to_string(),
            "u32 : :: ormlite :: decode :: Decode < 'a , R :: Database > , ".to_owned()
                + "u32 : :: ormlite :: types :: Type < R :: Database > , "
                + "String : :: ormlite :: decode :: Decode < 'a , R :: Database > , "
                + "String : :: ormlite :: types :: Type < R :: Database > , "
                + "bool : :: ormlite :: decode :: Decode < 'a , R :: Database > , "
                + "bool : :: ormlite :: types :: Type < R :: Database > ,"
        );
    }
}
