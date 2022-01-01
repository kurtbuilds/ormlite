use derive_builder::Builder;
use proc_macro2::Span;
use syn::{Lit, Meta, NestedMeta, Type};

#[derive(Builder, Debug)]
#[builder(field(public))]
pub struct TableAttr {
    pub table_name: String,
    pub primary_key_column: String,
    pub columns: Vec<Column>,
    pub insert_struct_name: String,
}

#[derive(Clone, Debug)]
pub struct Column {
    pub column_name: String,
    pub column_type: Type,
    pub primary_key: bool,
}

pub(crate) fn extract_meta(
    attrs: &[syn::Attribute],
) -> impl Iterator<Item = (String, syn::Lit)> + '_ {
    attrs
        .iter()
        .filter(|a| a.path.is_ident("ormlite"))
        .flat_map(|a| {
            let meta = a.parse_meta().unwrap();
            match meta {
                Meta::List(syn::MetaList { nested, .. }) => nested.into_iter(),
                _ => panic!("attribute must be a list"),
            }
        })
        .map(|nested| match nested {
            NestedMeta::Meta(m) => m,
            NestedMeta::Lit(_l) => unimplemented!(),
        })
        .map(|m| match m {
            Meta::NameValue(syn::MetaNameValue { path, lit, .. }) => {
                (path.get_ident().unwrap().to_string(), lit)
            }
            _ => panic!("attribute must be a name value"),
        })
}

pub(crate) fn parse_attrs(attrs: &[syn::Attribute], _span: Span) -> syn::Result<TableAttrBuilder> {
    let mut attr_owned = TableAttrBuilder::default();
    let z = &mut attr_owned;
    extract_meta(attrs)
        .map(|(ident, lit)| match ident.as_str() {
            "table_name" => match lit {
                Lit::Str(l) => {
                    z.table_name(l.value());
                }
                _ => panic!("table_name must be a string literal"),
            },
            // "primary_key" => match lit {
            //     Lit::Str(l) => {
            //         z.primary_key_column(l.value());
            //     }
            //     _ => panic!("primary_key_column must be a string literal"),
            // },
            "insert" => match lit {
                Lit::Verbatim(l) => {
                    println!("{}", l);
                    todo!();
                    // z.insert_struct_name(l.
                    // z.primary_key_column(l.value());
                }
                _ => panic!("primary_key_column must be a string literal"),
            },
            _ => panic!("Unexpected attribute: {}", ident),
        })
        .for_each(|_| {});
    Ok(attr_owned)
}
