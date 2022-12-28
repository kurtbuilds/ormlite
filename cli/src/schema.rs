use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;
use ignore::Walk;
use sqldiff::Schema;
use syn::Item;

pub trait TryFromOrmlite: Sized {
    fn try_from_ormlite_project(path: &Path) -> Result<Self>;
}

impl TryFromOrmlite for Schema {
    fn try_from_ormlite_project(path: &Path) -> Result<Self> {
        let walk = Walk::new(path)
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|e| e == "rs")
                .unwrap_or(false));
        let mut schema = Schema::new();
        for entry in walk {
            let contents = fs::read_to_string(&entry.path())?;
            if !contents.contains("Model") {
                continue;
            }
            let mut ast = syn::parse_file(&contents)?;
            let Some(first) = ast.items.into_iter().filter(|item| matches!(item, Item::Struct(_))).next() else {
                continue;
            };
            let Item::Struct(s) = first else { panic!() };
            let attributes = crate::syndecode::Attributes::from(&s.attrs);
            println!("struct: {:?}", s.ident);
            println!("attributes: {:?}", attributes);
            break
            // ast.items.retain(|item| {
            //     match item {
            //         Item::Struct(item) => {
            //             println!("{}", entry.path().display());
            //             println!("struct: {:?}", item.ident);
            //             println!("struct: {:#?}", item.attrs);
            //             let attributes = Attributes::from(&item.attrs);
            //             println!("attributes: {:?}", attributes);
            //             false
            //         }
            //         _ => false
            //     }
            // });
        }
        Ok(Self {
            tables: vec![],
        })
    }
}