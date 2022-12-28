#![allow(unused)]
use clap::{Parser, Subcommand};
use anyhow::Result;
mod schema;
mod command;
mod util;
mod syndecode;

use command::*;


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Migrate(Migrate),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Migrate(m) => m.run()
    }
}