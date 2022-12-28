#![allow(unused)]
extern crate core;

use clap::{Parser, Subcommand};
use anyhow::Result;
mod schema;
mod command;
mod util;
mod syndecode;
pub(crate) mod config;

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
    Up(Up),
    Down(Down),
}

fn main() -> Result<()> {
    use Command::*;
    let cli = Cli::parse();
    match cli.command {
        Migrate(m) => m.run(),
        Up(up) => up.run(),
        Down(down) => down.run(),
    }
}