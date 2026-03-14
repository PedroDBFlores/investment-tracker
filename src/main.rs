mod cli;
mod core;
mod error;
mod utils;

use clap::Parser;
use cli::commands::Cli;
use error::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
