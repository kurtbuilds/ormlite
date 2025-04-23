use crate::command::{get_executed_migrations, get_pending_migrations, MigrationType};
use crate::util::{create_runtime, CommandSuccess};
use anyhow::{anyhow, Result};
use clap::Parser;
use ormlite::postgres::{PgArguments, PgConnection};
use ormlite::{Acquire, Arguments, Connection, Executor};
use ormlite_core::config::{get_var_database_url, get_var_migration_folder, get_var_snapshot_folder};
use sha2::{Digest, Sha384};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::time::Instant;
use tracing::debug;

#[derive(Parser, Debug)]
pub struct Up {
    #[clap(long, short)]
    /// Run all pending migrations. If not set, only the first pending migration will be run.
    all: bool,

    #[clap(long, short)]
    /// Only affects `up` command when using simple (up-only) migrations. Causes command to skip creating the backup.
    no_snapshot: bool,

    #[clap(long, short)]
    /// Only affects `up` command when using up/down migrations. Causes command to create a backup.
    snapshot: bool,
}

impl Up {
    pub fn run(self) -> Result<()> {
        let folder = get_var_migration_folder();
        let runtime = create_runtime();
        let url = get_var_database_url();
        let mut conn = runtime.block_on(PgConnection::connect(&url))?;
        let conn = runtime.block_on(conn.acquire()).unwrap();

        let executed = runtime.block_on(get_executed_migrations(conn))?;
        let pending = get_pending_migrations(&folder)
            .unwrap()
            .into_iter()
            .filter(|m| m.migration_type() != MigrationType::Down)
            .collect::<Vec<_>>();
        for e in &executed {
            debug!("Executed: {}", e.full_name());
        }
        debug!("{} migrations executed", executed.len());
        for p in &pending {
            debug!("Pending: {}", p.full_name());
        }
        debug!("{} migrations pending", pending.len());

        let pending_hashset = pending.iter().map(|m| m.version).collect::<HashSet<_>>();
        for e in &executed {
            if !pending_hashset.contains(&e.version) {
                return Err(anyhow!("Migration {} was executed on the database, but was not found in your migrations folder. Your migrations are out of sync.", e.full_name()));
            }
        }
        if executed.len() == pending.len() {
            eprintln!("No migrations to run.");
            return Ok(());
        }
        let last_executed = executed.last().map(|m| m.name.clone()).unwrap_or("0_empty".to_string());
        let executed = executed.into_iter().map(|m| m.version).collect::<HashSet<_>>();

        let pending = pending
            .into_iter()
            .filter(|m| !executed.contains(&m.version))
            .collect::<Vec<_>>();

        let is_simple = pending.last().as_ref().unwrap().migration_type() == MigrationType::Simple;
        if (is_simple && !self.no_snapshot) || (!is_simple && self.snapshot) {
            eprintln!("Creating snapshot...");
            let snapshot_folder = get_var_snapshot_folder();
            fs::create_dir_all(&snapshot_folder).unwrap();
            let file_path = snapshot_folder.join(format!("{last_executed}.sql.bak"));
            let backup_file = File::create(&file_path)?;
            std::process::Command::new("pg_dump")
                .arg(&url)
                .stdout(backup_file)
                .output()?
                .ok_or("Failed to create backup")?;
            eprintln!("{}: Created database snapshot.", file_path.display());
        }

        let pending = pending.iter().take(if self.all { pending.len() } else { 1 });
        for migration in pending {
            debug!("Running migration: {}", migration.name);
            let file_path = folder
                .join(&migration.name)
                .with_extension(migration.migration_type().extension());
            let body = fs::read_to_string(&file_path)?;

            let checksum = Sha384::digest(body.as_bytes()).to_vec();

            let start = Instant::now();
            runtime
                .block_on(conn.execute(&*body))
                .map_err(|e| anyhow!("Error while running migration {}: {}", &migration.name, e))?;
            let elapsed = start.elapsed();

            let mut args = PgArguments::default();
            args.add(migration.version).map_err(|e| anyhow!(e))?;
            args.add(&migration.description).map_err(|e| anyhow!(e))?;
            args.add(checksum).map_err(|e| anyhow!(e))?;
            args.add(elapsed.as_nanos() as i64).map_err(|e| anyhow!(e))?;
            let q = ormlite::query_with("INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) VALUES ($1, $2, NOW(), true, $3, $4)", args);
            runtime.block_on(q.execute(&mut *conn))?;
            eprintln!("{}: Executed migration", file_path.display());
        }
        Ok(())
    }
}
