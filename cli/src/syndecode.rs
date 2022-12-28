use proc_macro2::TokenTree;
use self::args::ArgsAttribute;
use self::derive::DeriveAttribute;
use self::properties::PropertiesAttribute;

mod args;
mod derive;
mod properties;
mod error;

#[derive(Debug)]
pub enum Attribute {
    Derive(DeriveAttribute),
    Properties(PropertiesAttribute),
    Args(ArgsAttribute),
    Flag(String),
}

impl Attribute {
    pub fn name(&self) -> &str {
        match self {
            Attribute::Derive(attr) => attr.trait_name(),
            Attribute::Properties(attr) => attr.name.as_str(),
            Attribute::Args(attr) => attr.name.as_str(),
            Attribute::Flag(attr) => &attr,
        }
    }
}

#[derive(Debug)]
pub struct Attributes(Vec<Attribute>);

impl From<&Vec<syn::Attribute>> for Attributes {
    fn from(attrs: &Vec<syn::Attribute>) -> Self {
        let mut attributes = Vec::new();
        for attr in attrs.iter() {
            if attr.path.is_ident("derive") {
                DeriveAttribute::decode_many_from_attr(attr)
                    .into_iter()
                    .map(|d| Attribute::Derive(d))
                    .for_each(|a| attributes.push(a));
            } else {
                let name = attr.path.segments.first().unwrap().ident.to_string();
                let attr = if attr.tokens.is_empty() {
                    Attribute::Flag(name)
                } else {
                    PropertiesAttribute::try_from(attr).map(|p| Attribute::Properties(p))
                        .or_else(|_| {
                            ArgsAttribute::try_from(attr).map(|a| Attribute::Args(a))
                        }).expect("Unknown attribute")
                };
                attributes.push(attr);
            }
        }
        println!("attr: {:#?}", attributes);
        Attributes(attributes)
    }
}
