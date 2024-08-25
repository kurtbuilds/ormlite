use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Meta, Token};

/// When doing static analysis, `structmeta` and `darling` do not parse attributes
/// in a cfg_attr, so we have this struct to enable that
pub struct CfgAttr {
    // pub _condition: Meta,
    pub attrs: Punctuated<Meta, Token![,]>,
}

impl Parse for CfgAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _condition: Meta = input.parse()?;
        let _: Comma = input.parse()?;
        let attrs = input.parse_terminated(Meta::parse, Token![,])?;
        Ok(CfgAttr { attrs })
    }
}
