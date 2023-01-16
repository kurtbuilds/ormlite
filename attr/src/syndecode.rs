use self::args::ArgsAttribute;
use self::derive::DeriveAttribute;
use self::properties::PropertiesAttribute;

mod args;
mod derive;
mod properties;

#[derive(Debug)]
pub enum Attribute {
    Derive(DeriveAttribute),
    Properties(PropertiesAttribute),
    Args(ArgsAttribute),
    Flag(String),
}

// impl Attribute {
//     pub fn name(&self) -> &str {
//         match self {
//             Attribute::Derive(attr) => attr.trait_name(),
//             Attribute::Properties(attr) => attr.name.as_str(),
//             Attribute::Args(attr) => attr.name.as_str(),
//             Attribute::Flag(attr) => &attr,
//         }
//     }
// }

#[derive(Debug)]
pub struct Attributes(Vec<Attribute>);

impl Attributes {
    pub fn has_derive(&self, trait_name: &str) -> bool {
        self.0.iter().any(|attr| {
            matches!(attr, Attribute::Derive(a) if a.trait_name() == trait_name)
        })
    }

    pub fn derives(&self) -> Vec<&str> {
        self.0.iter().filter_map(|attr| match attr {
            Attribute::Derive(a) => Some(a.trait_name()),
            _ => None
        }).collect::<Vec<_>>()
    }
}

impl Attributes {
    pub(crate) fn filter_from(attrs: &[syn::Attribute], filter: &str) -> Self {
        let mut attributes = Vec::new();
        for attr in attrs.iter() {
            if attr.path.is_ident("derive") {
                DeriveAttribute::decode_many_from_attr(attr)
                    .into_iter()
                    .map(Attribute::Derive)
                    .for_each(|a| attributes.push(a));
            } else {
                let name = attr.path.segments.last().unwrap().ident.to_string();
                if name != filter {
                    continue
                }
                let attr = if attr.tokens.is_empty() {
                    Attribute::Flag(name)
                } else {
                    PropertiesAttribute::try_from(attr).map(Attribute::Properties)
                        .or_else(|_| {
                            ArgsAttribute::try_from(attr).map(Attribute::Args)
                        }).expect("Unknown attribute")
                };
                attributes.push(attr);
            }
        }
        Attributes(attributes)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derives() {
        let attrs = Attributes(vec![
            Attribute::Derive(DeriveAttribute::new("ormlite::Model")),
        ]);
        assert!(attrs.has_derive("Model"));
    }
}