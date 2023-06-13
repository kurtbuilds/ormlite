use crate::command::{get_executed_migrations, get_pending_migrations, MigrationType};
use crate::util::{create_connection, create_runtime, CommandSuccess};
use anyhow::Result;
use clap::Parser;
use ormlite::postgres::PgArguments;
use ormlite::{Acquire, Arguments, Executor};
use ormlite_core::config::{
    get_var_database_url, get_var_migration_folder, get_var_snapshot_folder,
};
use sha2::{Digest, Sha384};
use std::fs;
use std::fs::File;
use std::time::Instant;

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
        let mut conn = create_connection(&url, &runtime).unwrap();
        let conn = runtime.block_on(conn.acquire()).unwrap();

        let executed = get_executed_migrations(&runtime, &mut *conn).unwrap();
        let pending = get_pending_migrations(&folder)
            .unwrap()
            .into_iter()
            .filter(|m| m.migration_type() != MigrationType::Down)
            .collect::<Vec<_>>();

        if executed.len() >= pending.len() {
            eprintln!("No migrations to run.");
            return Ok(());
        }

        if (pending.last().as_ref().unwrap().migration_type() == MigrationType::Simple
            && !self.no_snapshot)
            || (pending.last().as_ref().unwrap().migration_type() != MigrationType::Simple
                && self.snapshot)
        {
            eprintln!("Creating snapshot...");
            let snapshot_folder = get_var_snapshot_folder();
            fs::create_dir_all(&snapshot_folder).unwrap();
            let file_stem = executed
                .last()
                .map(|m| m.name.clone())
                .unwrap_or("0_empty".to_string());
            let file_path = snapshot_folder.join(format!("{file_stem}.sql.bak"));
            let backup_file = File::create(&file_path)?;
            std::process::Command::new("pg_dump")
                .arg(&url)
                .stdout(backup_file)
                .output()?
                .ok_or("Failed to create backup")?;
            eprintln!("{}: Created database snapshot.", file_path.display());
        }

        let iter =
            pending
                .iter()
                .skip(executed.len())
                .take(if self.all { pending.len() } else { 1 });
        for migration in iter {
            let file_path = folder
                .join(&migration.name)
                .with_extension(migration.migration_type().extension());
            let body = std::fs::read_to_string(&file_path)?;

            let checksum = Sha384::digest(body.as_bytes()).to_vec();

            let start = Instant::now();
            runtime.block_on(conn.execute(&*body))?;
            let elapsed = start.elapsed();

            let mut args = PgArguments::default();
            args.add(migration.version);
            args.add(&migration.description);
            args.add(checksum);
            args.add(elapsed.as_nanos() as i64);
            let q = ormlite::query_with("INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) VALUES ($1, $2, NOW(), true, $3, $4)", args);
            runtime.block_on(q.execute(&mut *conn))?;
            eprintln!("{}: Executed migration", file_path.display());
        }
        Ok(())
    }
}
