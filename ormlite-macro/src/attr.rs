use derive_builder::Builder;
use proc_macro2::Span;
use structmeta::StructMeta;
use syn::{Ident, Lit, LitStr, Meta, NestedMeta, Type};

#[derive(StructMeta, Debug)]
pub struct ModelAttributes {
    pub table: Option<LitStr>,
    pub insert: Option<Ident>,
}

#[derive(StructMeta, Debug)]
pub struct ColumnAttributes {
    pub primary_key: bool,
}

#[derive(Builder, Debug)]
#[builder(field(public))]
pub struct TableMeta {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Vec<ColumnMeta>,
    pub insert_struct: Option<String>,
}

#[derive(Clone, Debug, Builder)]
#[builder(field(public))]
pub struct ColumnMeta {
    pub column_name: String,
    pub column_type: Type,
    pub marked_primary_key: bool,
}

// pub(crate) fn extract_meta(
//     attrs: &[syn::Attribute],
// ) -> impl Iterator<Item = (String, Option<syn::Lit>)> + '_ {
//     attrs
//         .iter()
//         .filter(|a| a.path.is_ident("ormlite"))
//         .flat_map(|a| {
//             let meta = a.parse_meta().unwrap();
//             match meta {
//                 Meta::List(syn::MetaList { nested, .. }) => nested.into_iter(),
//                 _ => panic!("attribute must be a list"),
//             }
//         })
//         .map(|nested| match nested {
//             NestedMeta::Meta(m) => {
//                 eprintln!("nested meta: {:?}", m);
//                 m
//             }
//             NestedMeta::Lit(_l) => panic!("literal not supported"),
//         })
//         .map(|m| match m {
//             Meta::NameValue(syn::MetaNameValue { path, lit, .. }) => {
//                 (path.get_ident().unwrap().to_string(), Some(lit))
//             }
//             Meta::Path(syn::Path { segments, .. }) => match segments.iter().next().unwrap() {
//                 syn::PathSegment { ident, .. } => (ident.to_string(), None),
//             },
//             _ => panic!("attribute must be a name value. Found: {:?}", m),
//         })
// }

// pub(crate) fn parse_attrs(attrs: &[syn::Attribute], _span: Span) -> syn::Result<TableMetaBuilder> {
//     let mut attr_owned = TableMetaBuilder::default();
//     let z = &mut attr_owned;
//     extract_meta(attrs)
//         .map(|(ident, lit)| match ident.as_str() {
//             "table" => match lit.unwrap() {
//                 Lit::Str(l) => {
//                     z.table_name(l.value());
//                 }
//                 _ => panic!("table_name must be a string literal"),
//             },
//             // "primary_key" => match lit {
//             //     Lit::Str(l) => {
//             //         z.primary_key_column(l.value());
//             //     }
//             //     _ => panic!("primary_key_column must be a string literal"),
//             // },
//             "insert" => match lit.unwrap() {
//                 Lit::Verbatim(l) => {
//                     println!("found insert: {}", l);
//                     todo!();
//                     // z.insert_struct_name(l.
//                     // z.primary_key_column(l.value());
//                 }
//                 _ => panic!("insert must be a string literal"),
//             },
//             _ => panic!("Unexpected attribute: {}", ident),
//         })
//         .for_each(|_| {});
//     Ok(attr_owned)
// }
