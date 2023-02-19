use syn::__private::quote::__private::TokenTree;

#[derive(Debug)]
pub struct DeriveAttribute(String);

impl DeriveAttribute {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    // /// Get the full name, including a module path.
    // fn full_name(&self) -> &str {
    //     &self.0
    // }

    /// Get only the trait name
    pub(crate) fn trait_name(&self) -> &str {
        if self.0.contains("::") {
            self.0.rsplit("::").next().unwrap()
        } else {
            &self.0
        }
    }

    pub fn decode_many_from_attr(attr: &syn::Attribute) -> Vec<Self> {
        let group = attr.tokens.clone().into_iter().next().expect("Derive attribute must have a token group");
        let TokenTree::Group(group) = group else {
            panic!("Derive attribute must have a token group");
        };
        let group = group.stream().into_iter();

        let mut current: Option<String> = None;
        let mut attributes = Vec::new();

        for tok in group {
            match tok {
                TokenTree::Group(_) => {}
                TokenTree::Ident(i) => {
                    if let Some(ref mut current) = current {
                        current.push_str("::");
                        current.push_str(&i.to_string());
                    } else {
                        current = Some(i.to_string());
                    }
                }
                TokenTree::Punct(p) if p.as_char() == ',' => {
                    if let Some(name) = current.take() {
                        attributes.push(Self(name));
                    }
                }
                _ => {}
            }
        }
        attributes
    }
}
