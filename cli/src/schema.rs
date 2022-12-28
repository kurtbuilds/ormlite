use std::path::Path;
use anyhow::Result;
use ignore::Walk;
use sqldiff::Schema;

pub trait TryFromOrmlite: Sized {
    fn try_from_ormlite_project(path: &Path) -> Result<Self>;
}

impl TryFromOrmlite for Schema {
    fn try_from_ormlite_project(path: &Path) -> Result<Self> {
        for entry in Walk::new(path).filter_map(|e| e.ok()) {
            println!("{}", entry.path().display());
        }
        Ok(Self {
            tables: vec![],
        })
    }
}