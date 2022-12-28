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
use ormlite::Acquire;

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

        if !self.empty {
            let url = var("DATABASE_URL").expect("DATABASE_URL must be set");
            let mut current: Schema = runtime.block_on(async {
                let pool = ormlite::PgPoolOptions::new()
                    .acquire_timeout(std::time::Duration::from_secs(1))
                    .connect(&url)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to connect to database. Is the database running?"))
                    ?;
                // let pool = ormlite::PgPool::connect_with(ormlite::PgConnectOptions::).await?;
                let mut conn = pool.acquire().await?;
                let conn = conn.acquire().await?;
                let schema = Schema::try_from_database(conn, "public").await?;
                Ok::<_, anyhow::Error>(schema)
            }).context("Failed to connect to database.")?;

            current.tables.retain(|t| t.name != "_sqlx_migrations");

            let desired = Schema::try_from_ormlite_project(Path::new("."))?;
            println!("Current:\n{:#?}\nDesired:\n{:#?}", current, desired);
            let migration = current.migrate_to(desired, &sqldiff::Options::default())?;
            println!("{:#?}", migration);
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