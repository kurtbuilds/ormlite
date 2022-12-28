/// Decode a sqldiff::Schema from the current code base.
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;
use ignore::Walk;
use sqldiff::Schema;
use syn::Item;
use crate::syndecode::Attributes;

pub trait TryFromOrmlite: Sized {
    fn try_from_ormlite_project(path: &Path) -> Result<Self>;
}

impl TryFromOrmlite for Schema {
    fn try_from_ormlite_project(path: &Path) -> Result<Self> {
        let walk = Walk::new(path)
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|e| e == "rs")
                .unwrap_or(false));

        let mut schema = Self::new();

        for entry in walk {
            let contents = fs::read_to_string(&entry.path())?;
            if !contents.contains("Model") {
                continue;
            }
            let mut ast = syn::parse_file(&contents)?;
            let structs = ast.items.into_iter().filter_map(|item| match item {
                Item::Struct(s) => Some(s),
                _ => None,
            })
                .map(|s| {
                    let attrs = Attributes::from(&s.attrs);
                    (s, attrs)
                })
                .filter(|(_, attrs)| !attrs.derives("Model"))
                .collect::<Vec<_>>();
            for (item, attrs) in structs {
            }
        }
        Ok(schema)
    }
}