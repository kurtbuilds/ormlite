use quote::ToTokens;
use structmeta::StructMeta;
use syn::Path;

#[derive(StructMeta)]
pub struct Repr(
    #[struct_meta(unnamed)]
    Path,
);

impl std::fmt::Debug for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_token_stream().to_string().fmt(f)
    }
}

impl std::fmt::Display for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_token_stream().to_string().fmt(f)
    }
}

impl PartialEq<&str> for Repr {
    fn eq(&self, &other: &&str) -> bool {
        self.0.is_ident(other)
    }
}

impl Repr {
    const ATTRIBUTE: &'static str = "repr";

    pub fn from_attributes(attrs: &[syn::Attribute]) -> Option<Self> {
        for a in attrs {
            let Some(ident) = a.path().get_ident() else { continue; };
            if ident == Self::ATTRIBUTE {
                // semantically, the parse error and returning Some are different,
                // so we're writing out Some() instead of using `.ok()`
                return Some(a.parse_args().unwrap());
            }
        }
        None
    }
}