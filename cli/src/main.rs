use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Migrate {
        name: String,

        /// Create an empty migration, don't attempt to infer changes
        #[clap(long)]
        empty: bool,
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("{:?}", cli);
    Ok(())
}