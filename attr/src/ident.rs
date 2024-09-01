//! This ident needs to exist because the proc_macro2 idents are not Send.
use proc_macro2::TokenStream;
use quote::TokenStreamExt;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ident(String);

impl Ident {
    pub fn as_ref(&self) -> &String {
        &self.0
    }
}

impl From<&proc_macro2::Ident> for Ident {
    fn from(ident: &proc_macro2::Ident) -> Self {
        Ident(ident.to_string())
    }
}

impl From<&str> for Ident {
    fn from(ident: &str) -> Self {
        Ident(ident.to_string())
    }
}

impl From<String> for Ident {
    fn from(ident: String) -> Self {
        Ident(ident)
    }
}

impl From<&String> for Ident {
    fn from(ident: &String) -> Self {
        Ident(ident.clone())
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

impl<T> PartialEq<T> for Ident
where
    T: AsRef<str>,
{
    fn eq(&self, t: &T) -> bool {
        let t = t.as_ref();
        self.0.as_str() == t
    }
}
