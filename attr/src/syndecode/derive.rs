use proc_macro2::TokenTree;

#[derive(Debug)]
pub struct DeriveAttribute(String);

impl DeriveAttribute {
    #[allow(dead_code)]
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
        let tokens = attr.meta.require_list().expect("Derive attribute must have a token group").clone().tokens;
        let mut current: Option<String> = None;
        let mut attributes = Vec::new();

        for tok in tokens {
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
        if let Some(current) = current {
            attributes.push(Self(current));
        }
        attributes
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_single_attr() {
        let attr = syn::parse_quote! {
            #[derive(Model)]
        };
        let attrs = DeriveAttribute::decode_many_from_attr(&attr);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].trait_name(), "Model");
    }

    #[test]
    fn test_many_from_attr() {
        let attr = syn::parse_quote! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        };
        let attrs = DeriveAttribute::decode_many_from_attr(&attr);
        assert_eq!(attrs.len(), 5);
        assert_eq!(attrs[0].trait_name(), "Debug");
        assert_eq!(attrs[1].trait_name(), "Clone");
        assert_eq!(attrs[2].trait_name(), "PartialEq");
        assert_eq!(attrs[3].trait_name(), "Eq");
        assert_eq!(attrs[4].trait_name(), "Hash");
    }
}
