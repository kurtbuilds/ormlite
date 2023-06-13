pub use self::args::ArgsAttribute;
use self::derive::DeriveAttribute;
use self::properties::PropertiesAttribute;
use std::fmt::Display;

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

impl Attributes {
    pub fn has_derive(&self, trait_name: &str) -> bool {
        self.0
            .iter()
            .any(|attr| matches!(attr, Attribute::Derive(a) if a.trait_name() == trait_name))
    }

    pub fn iter_args(&self) -> impl Iterator<Item = &ArgsAttribute> {
        self.0.iter().filter_map(|attr| match attr {
            Attribute::Args(a) => Some(a),
            _ => None,
        })
    }
}

impl Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        if self
            .0
            .iter()
            .filter(|f| matches!(f, Attribute::Derive(_)))
            .count()
            > 0
        {
            write!(f, "#[derive(")?;
            for attr in self.0.iter().filter_map(|f| match f {
                Attribute::Derive(d) => Some(d),
                _ => None,
            }) {
                write!(f, "{},", attr.trait_name())?;
            }
            write!(f, ")], ")?;
        }
        for attr in self.0.iter() {
            match attr {
                Attribute::Derive(_) => {}
                Attribute::Properties(attr) => write!(f, "{}({:?}), ", attr.name, attr.properties)?,
                Attribute::Args(attr) => write!(f, "{}({}), ", attr.name, attr.args.join(", "))?,
                Attribute::Flag(attr) => write!(f, "{}, ", attr)?,
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl From<&[syn::Attribute]> for Attributes {
    fn from(attrs: &[syn::Attribute]) -> Self {
        let mut attributes = Vec::new();
        for attr in attrs.iter() {
            if attr.path.is_ident("derive") {
                DeriveAttribute::decode_many_from_attr(attr)
                    .into_iter()
                    .map(Attribute::Derive)
                    .for_each(|a| attributes.push(a));
            } else {
                let name = attr.path.segments.last().unwrap().ident.to_string();
                let attr = if attr.tokens.is_empty() {
                    Attribute::Flag(name)
                } else {
                    PropertiesAttribute::try_from(attr)
                        .map(Attribute::Properties)
                        .or_else(|_| ArgsAttribute::try_from(attr).map(Attribute::Args))
                        .expect("Unknown attribute")
                };
                attributes.push(attr);
            }
        }
        Attributes(attributes)
    }
}

impl Attributes {
    /// keeps all the derives, but filters the attributes for the word
    pub fn retain(&mut self, filter: &str) {
        self.0
            .retain(|f| matches!(f, Attribute::Derive(_)) || f.name() == filter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derives() {
        let attrs = Attributes(vec![Attribute::Derive(DeriveAttribute::new(
            "ormlite::Model",
        ))]);
        assert!(attrs.has_derive("Model"));
    }

    #[test]
    fn test_repr() {
        let q = quote::quote! {
            #[derive(sqlx::Type)]
            #[repr(u8)]
            pub enum Foo {
                Bar,
                Baz,
            }
        };
        let item = syn::parse2::<syn::ItemEnum>(q).unwrap();
        let attrs = Attributes::from(item.attrs.as_slice());
        assert!(attrs.has_derive("Type"));
        assert_eq!(attrs.iter_args().count(), 1);
        let args = attrs.iter_args().next().unwrap();
        assert_eq!(args.name, "repr");
        assert_eq!(args.args, vec!["u8"]);
    }
}
