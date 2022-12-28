use std::env::var;
use std::fs;
use time::{OffsetDateTime as DateTime};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use clap::Parser;
use anyhow::{Context, Result};
use sqldiff::Schema;
use crate::schema::TryFromOrmlite;

const MIGRATION_FOLDER: &str = "migrations";


enum MigrationType {
    Simple,
    Up,
    Down,
}

#[derive(Parser, Debug)]
pub struct Migrate {
    name: String,

    /// Create an empty migration, don't attempt to infer changes
    #[clap(long)]
    empty: bool,

    /// Create an empty migration, don't attempt to infer changes
    #[clap(long, short)]
    reversible: bool,
}

fn create_migration(path: &Path, mut file_name: String, migration: MigrationType) -> Result<()> {
    match migration {
        MigrationType::Simple => file_name.push_str(".sql"),
        MigrationType::Up => file_name.push_str(".up.sql"),
        MigrationType::Down => file_name.push_str(".down.sql"),
    }

    let path = path.join(&file_name);

    let mut file = File::create(path).context("Failed to create file")?;
    file.write_all(b"-- Add migration script here")
        .context("Could not write to file")?;
    Ok(())
}

impl Migrate {
    pub fn run(&self) -> Result<()> {
        let runtime = crate::util::runtime();

        let folder = var("MIGRATION_FOLDER").unwrap_or_else(|_| MIGRATION_FOLDER.to_string());
        let folder = PathBuf::from_str(&folder).unwrap();

        println!("migrate: {:?}", self);

        if !self.empty {

            let url = var("DATABASE_URL").expect("DATABASE_URL must be set");
            let pool = runtime.block_on(ormlite::PgPool::connect(&url)).context("Failed to connect to database.")?;

            let current = runtime.block_on(Schema::try_from_database(pool, "public"))?;
            let desired = Schema::try_from_ormlite_project(Path::new("."))?;
        }



        // fs::create_dir_all(&folder).context("Unable to create migrations directory")?;
        //
        // let dt = Utc::now();
        // let mut file_name = dt.format("%Y%m%d%H%M%S").to_string();
        // file_name.push_str("_");
        // file_name.push_str(&self.name);
        // if self.reversible {
        //     create_migration(&folder, file_name.clone(), MigrationType::Up)?;
        //     create_migration(&folder, file_name.clone(), MigrationType::Down)?;
        // } else {
        //     create_migration(&folder, file_name.clone(), MigrationType::Simple)?;
        // }
        // println!("{}: Generated migration.", file_name);
        Ok(())
    }
}