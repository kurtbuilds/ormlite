use proc_macro2::TokenStream;
use quote::TokenStreamExt;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ident(pub String);

impl Ident {
    pub fn new(ident: &str) -> Self {
        Ident(ident.to_string())
    }
    pub fn as_ref(&self) -> &String {
        &self.0
    }
}

impl From<&proc_macro2::Ident> for Ident {
    fn from(ident: &proc_macro2::Ident) -> Self {
        Ident(ident.to_string())
    }
}

impl quote::ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(proc_macro2::Ident::new(&self.0, proc_macro2::Span::call_site()))
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
