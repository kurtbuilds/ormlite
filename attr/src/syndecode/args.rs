use crate::SyndecodeError;
use proc_macro2::TokenTree;

#[derive(Debug)]
pub struct ArgsAttribute {
    pub name: String,
    pub args: Vec<String>,
}

impl TryFrom<&syn::Attribute> for ArgsAttribute {
    type Error = SyndecodeError;

    fn try_from(attr: &syn::Attribute) -> Result<Self, Self::Error> {
        let name = attr
            .path
            .segments
            .first()
            .ok_or_else(|| SyndecodeError("Must have a segment.".to_string()))?
            .ident
            .to_string();
        let group = attr.tokens.clone().into_iter().next().ok_or_else(|| {
            SyndecodeError("ArgAttributes must have at least one token group.".to_string())
        })?;
        let mut args = Vec::new();
        if let TokenTree::Group(group) = group {
            let group = group.stream().into_iter();

            let mut current: Option<String> = None;

            for tok in group {
                match tok {
                    TokenTree::Punct(p) if p.as_char() == ',' => {
                        if let Some(current) = current.take() {
                            args.push(current)
                        }
                    }
                    _ => {
                        if let Some(ref mut current) = current {
                            current.push_str(&tok.to_string());
                        } else {
                            current = Some(tok.to_string());
                        }
                    }
                }
            }

            if let Some(current) = current {
                args.push(current)
            }
        };

        Ok(Self { name, args })
    }
}
