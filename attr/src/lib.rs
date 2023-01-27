#![allow(non_snake_case)]

mod attr;
mod metadata;
mod error;
mod ext;
mod syndecode;

use std::fs;
use std::path::Path;
use syn::{DeriveInput, Item};
use ignore::Walk;

use syndecode::Attributes;
pub use metadata::*;
pub use attr::*;
pub use error::*;
pub use ext::*;

#[derive(Default, Debug)]
pub struct LoadOptions {
    pub verbose: bool,
}

pub fn load_from_project(paths: &[&Path], opts: &LoadOptions) -> anyhow::Result<Vec<TableMetadata>> {
    let walk = paths.iter().flat_map(Walk::new);
    let mut results = vec![];

    let walk = walk.filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|e| e == "rs")
            .unwrap_or(false));

    for entry in walk {
        let contents = fs::read_to_string(entry.path())?;
        if !contents.contains("Model") {
            continue;
        }
        if opts.verbose {
            eprintln!("{}: Checking for #[derive(Model)]", entry.path().display());
        }
        let ast = syn::parse_file(&contents)?;
        let structs = ast.items.into_iter().filter_map(|item| match item {
            Item::Struct(s) => Some(s),
            _ => None,
        })
            .map(|s| {
                let attrs = Attributes::filter_from(&s.attrs, "ormlite");
                (s, attrs)
            })
            .inspect(|(s, attrs)| {
                if opts.verbose {
                    eprintln!("{}: Found struct {}. Detected derives: {:?}", entry.path().display(), s.ident, attrs.derives());
                }
            })
            .filter(|(_, attrs)| attrs.has_derive("Model"))
            .collect::<Vec<_>>();
        for (item, _attrs) in structs {
            let derive: DeriveInput = item.into();
            let table = TableMetadata::try_from(&derive)
                .map_err(|e| SyndecodeError(format!(
                    "{}: Encountered an error while scanning for #[derive(Model)] structs: {}",
                    entry.path().display(), e))
                )?;
            results.push(table);
        }
    }
    Ok(results)
}