use crate::SyndecodeError;
use proc_macro2::TokenTree;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PropertiesAttribute {
    pub name: String,
    pub properties: HashMap<String, String>,
}

impl TryFrom<&syn::Attribute> for PropertiesAttribute {
    type Error = SyndecodeError;

    fn try_from(attr: &syn::Attribute) -> Result<Self, Self::Error> {
        let name = attr
            .path
            .segments
            .first()
            .ok_or_else(|| SyndecodeError("Must have a segment.".to_string()))?
            .ident
            .to_string();
        let group = attr
            .tokens
            .clone()
            .into_iter()
            .next()
            .ok_or_else(|| SyndecodeError("Must have a token group.".to_string()))?;
        let TokenTree::Group(group) = group else {
            return Err(SyndecodeError("Must have a token group.".to_string()));
        };
        let mut group = group.stream().into_iter();

        let mut current_key: Option<String> = None;
        let mut current_value: Option<String> = None;
        let mut properties = HashMap::new();

        while let Some(tok) = group.next() {
            match tok {
                TokenTree::Punct(p) if p.as_char() == ',' => {
                    if let Some((key, value)) = current_key.take().zip(current_value.take()) {
                        properties.insert(key, value);
                    }
                }
                TokenTree::Ident(i) if current_key.is_none() => {
                    current_key = Some(i.to_string());
                    if !matches!(group.next(), Some(TokenTree::Punct(p)) if p.as_char() == '=') {
                        return Err(SyndecodeError("Expected an equals sign.".to_string()));
                    }
                }
                _ => {
                    if let Some(ref mut current_value) = current_value {
                        current_value.push_str(&tok.to_string());
                    } else {
                        current_value = Some(tok.to_string());
                    }
                }
            }
        }
        Ok(Self { name, properties })
    }
}
