use std::fs;
use std::fs::File;
use std::time::Instant;
use anyhow::Result;
use clap::Parser;
use ormlite::Acquire;
use ormlite::Arguments;
use ormlite::postgres::PgArguments;
use ormlite::Executor;
use crate::command::{get_executed_migrations, get_pending_migrations, MigrationType};
use crate::config::{get_var_snapshot_folder, get_var_database_url, get_var_migration_folder};
use crate::util::{CommandSuccess, create_connection, create_runtime};
use sha2::{Digest, Sha384};

const INIT_QUERY: &str = r#"
CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);
"#;

#[derive(Parser, Debug)]
pub struct Init {
}

impl Init {
    pub fn run(self) -> Result<()> {
        let runtime = create_runtime();
        let url = get_var_database_url();
        let mut conn_owned = create_connection(&url, &runtime)?;
        let conn = runtime.block_on(conn_owned.acquire())?;

        runtime.block_on(conn.execute(INIT_QUERY))?;

        Ok(())
    }
}