use clap::{Parser, Subcommand};
use anyhow::Result;

mod command;
mod util;

use command::*;


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a migration. Supports auto-generation based on models detected in the codebase.
    Migrate(Migrate),
    /// Run up migrations. Runs one migration, or pass `--all` to run all pending migrations.
    /// Optionally creates a snapshots to enable rollback, even when you only write an up migration.
    Up(Up),
    /// Run down migration. If no target revision is specified, the last migration will be run.
    Down(Down),
    /// Initiailize the database for use with `ormlite`. Creates the migrations table.
    Init(Init),
}

fn main() -> Result<()> {
    use Command::*;
    let cli = Cli::parse();
    match cli.command {
        Migrate(m) => m.run(),
        Up(up) => up.run(),
        Down(down) => down.run(),
        Init(init) => init.run(),
    }
}