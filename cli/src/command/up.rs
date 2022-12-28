use std::fs;
use std::fs::File;
use anyhow::Result;
use clap::Parser;
use sqlx::Acquire;
use ormlite::Executor;
use crate::command::{get_executed_migrations, get_pending_migrations, MigrationType};
use crate::config::{get_var_backup_folder, get_var_database_url, get_var_migration_folder};
use crate::util::{CommandSuccess, create_connection, create_runtime};

#[derive(Parser, Debug)]
pub struct Up {
    #[clap(long, short)]
    /// Run all pending migrations. If not set, only the first pending migration will be run.
    all: bool,

    #[clap(long, short)]
    /// Only affects `up` command when using simple (up-only) migrations. Causes command to skip creating the backup.
    no_backup: bool,

    #[clap(long, short)]
    /// Only affects `up` command when using up/down migrations. Causes command to create a backup.
    backup: bool,
}

impl Up {
    pub fn run(self) -> Result<()> {
        let folder = get_var_migration_folder();
        let runtime = create_runtime();
        let url = get_var_database_url();
        let mut conn = create_connection(&url, &runtime)?;
        let conn = runtime.block_on(conn.acquire())?;

        let executed = get_executed_migrations(&runtime, conn)?;
        let pending = get_pending_migrations(&folder)?
            .into_iter()
            .filter(|m| m.migration_type() != MigrationType::Down)
            .collect::<Vec<_>>();
        ;

        if executed.len() >= pending.len() {
            eprintln!("No migrations to run.");
            return Ok(());
        }

        if (pending.last().as_ref().unwrap().migration_type() == MigrationType::Simple && !self.no_backup) ||
            (pending.last().as_ref().unwrap().migration_type() != MigrationType::Simple && self.backup)
        {
            let backup_folder = get_var_backup_folder();
            fs::create_dir_all(&backup_folder)?;
            let file_stem = executed.last().map(|m| m.name.clone()).unwrap_or("0_empty".to_string());
            let file_path = backup_folder.join(format!("{}.sql.bak", file_stem));
            let backup_file = File::create(&file_path)?;
            std::process::Command::new("pg_dump")
                .arg(&url)
                .stdout(backup_file)
                .output()?
                .ok_or("Failed to create backup")?;
            eprintln!("{}: Created database snapshot.", file_path.display());
        }

        let iter = pending.iter().skip(executed.len()).take(if self.all { pending.len() } else { 1 });
        for migration in iter {
            let file_path = folder.join(&migration.name).with_extension(migration.migration_type().extension());
            let migration = std::fs::read_to_string(&file_path)?;
            runtime.block_on(conn.execute(&*migration))?;
            eprintln!("{}: Executed migration", file_path.display());
        }
        Ok(())
    }
}