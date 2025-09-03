//! # MuJoCo-rs-util
//! A CLI utility to support some development of MuJoCo-rs.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod views;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The command to execute
    #[command(subcommand)]
    command: Command
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    CreateViews {
        indexer_xmacro_path: PathBuf
    }
}




fn main() {
    let parser = Args::parse();
    use Command::*;
    match parser.command {
        CreateViews { indexer_xmacro_path } => {
            views::create_views(&indexer_xmacro_path);
        }
    }
}
