//! # MuJoCo-rs-util
//! A CLI utility to support some development of MuJoCo-rs.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod fixed_arr_fn;
mod model_fn;
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
    /// A helper to automatically create calls to macros to facilitate the
    /// the view to MjData/MjModel array.
    CreateViews {
        indexer_xmacro_path: PathBuf
    },
    /// Creates Rust wrappers around C MuJoCo functions that have
    /// fixed-sized arrays as parameters.
    CreateFixedArrayFunctionWrappers {
        mujoco_h_path: PathBuf
    },

    /// Creates method wrappers for functions that potentially and logically
    /// belong to MjModel.
    CreateModelMethods {
        mujoco_h_path: PathBuf
    }
}




fn main() {
    let parser = Args::parse(); 
    use Command::*;
    match parser.command {
        CreateViews { indexer_xmacro_path } => {
            views::create_views(&indexer_xmacro_path);
        },

        CreateFixedArrayFunctionWrappers { mujoco_h_path } => {
            fixed_arr_fn::create_fixed_array_fn_wrappers(&mujoco_h_path);
        },

        CreateModelMethods { mujoco_h_path } => {
            model_fn::create_model_methods(&mujoco_h_path);
        }
    }
}
