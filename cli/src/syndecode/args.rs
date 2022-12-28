use syn::__private::quote::__private::ext::RepToTokensExt;
use syn::__private::quote::__private::TokenTree;
use ormlite_attr::SyndecodeError;

#[derive(Debug)]
pub struct ArgsAttribute {
    pub name: String,
    pub args: Vec<String>,
}

impl TryFrom<&syn::Attribute> for ArgsAttribute {
    type Error = SyndecodeError;

    fn try_from(attr: &syn::Attribute) -> Result<Self, Self::Error> {
        let name = attr.path.segments.first().ok_or_else(|| SyndecodeError(format!("Must have a segment.")))?.ident.to_string();
        let group = attr.tokens.clone().into_iter().next().ok_or_else(|| SyndecodeError(format!("Must have a token group.")))?;
        let TokenTree::Group(group) = group else {
            return Err(SyndecodeError(format!("Must have a token group.")));
        };
        let mut group = group.stream().into_iter();

        let mut current: Option<String> = None;
        let mut args = Vec::new();

        while let Some(tok) = group.next() {
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
        Ok(Self {
            name,
            args,
        })
    }
}