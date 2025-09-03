//! # MuJoCo-rs-util
//! A CLI utility to support some development of MuJoCo-rs.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod getset;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The command to execute
    #[command(subcommand)]
    command: Command
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Creates getter and setter enums.
    GetSet {
        /// Path to the C file that contains the struct definition.
        path: PathBuf,

        /// The name of the struct to generate getters and setters for.
        struct_name: String,

        /// Accessor to the data. This is either `self` if the data is not wrapped,
        /// and ffi() if it is wrapped
        accessor: String
    }
}




fn main() {
    let parser = Args::parse();
    use Command::*;
    match parser.command {
        GetSet { path, struct_name, accessor} => {
            getset::create_impl_getter_setter(path.as_path(), &struct_name, &accessor);
        }
    }
}
