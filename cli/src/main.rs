use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod command;
mod util;

use command::*;


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[clap(long, short, global = true)]
    verbose: bool,
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
    let cli = Cli::parse();
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .without_time()
        )
        .with(tracing_subscriber::filter::Targets::new()
            .with_target(env!("CARGO_BIN_NAME"), level)
            .with_target("ormlite_attr", level)
            .with_target("sqlmo", level)
        )
        .init();
    use Command::*;
    match cli.command {
        Migrate(m) => m.run(),
        Up(up) => up.run(),
        Down(down) => down.run(),
        Init(init) => init.run(),
    }
}