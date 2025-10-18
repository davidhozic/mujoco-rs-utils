//! # MuJoCo-rs-util
//! A CLI utility to support some development of MuJoCo-rs.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod fixed_arr_fn;
mod array_slice;
mod model_fn;
mod typedef;
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
    /// belong to some MjX struct.
    CreateModelMethods {
        /// Path to the mujoco.h file.
        mujoco_h_path: PathBuf,
        /// The struct name to create method wrappers.
        struct_: String,
        
        /// Ignore the methods that contain these types in the parameters.
        #[arg(num_args=0..)]
        blacklist: Vec<String>
    },
    /// Create type redefinitions for types that start with a given string.
    CreateTypes {
        /// Path to the documentation APtypes.rst file
        api_reference: PathBuf,
        /// The prefix that the type needs to have.
        prefix: Option<String>,
    },
    /// Creates a `array_slice_dyn` macro call
    /// for generating a slice into the array.
    CreateArraySliceMacroCall {
        /// Path to the documentation structs.h file containing the documentation-public structs.
        structs_filepath: PathBuf,
        /// The prefix to add in front of the parsed length variable.
        accessor_prefix: String,
        /// The name of the struct for which to create the slice methods.
        struct_name: String
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

        CreateModelMethods { mujoco_h_path , struct_, blacklist} => {
            model_fn::create_mj_self_methods(&mujoco_h_path, &struct_, &blacklist);
        }

        CreateTypes { api_reference, prefix } => {
            typedef::create_types(&api_reference, prefix.as_deref());
        }

        CreateArraySliceMacroCall { structs_filepath, accessor_prefix, struct_name } => {
            array_slice::create_array_slice(&structs_filepath, &accessor_prefix, &struct_name);
        }
    }
}
