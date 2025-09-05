//! Module for the CreateFixedArrayFunctionWrappers command.

use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::fs;
use regex;


const RE_FUNCTION_DECL: &str = r"MJAPI\s+((?:const)?\s*(?:[A-z0-9_*]+))\s+(\w+)\s*\((.+)\)";


fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    first.to_uppercase().chain(chars).collect()
}


pub fn create_fixed_array_fn_wrappers(mujoco_h_path: &Path) {
    let filedata = fs::read_to_string(mujoco_h_path).expect("could not read the file");
        /* Parse declaration */
    let re = regex::Regex::new(RE_FUNCTION_DECL).unwrap();
    let mut parameter_parts: VecDeque<_>;
    let mut mutability;
    let mut position_start;
    let mut position_end;
    let mut parameter_end;
    let mut param_type;
    let mut parameter_arr_len;
    let mut parameters_joined;
    let mut return_type_out;
    let mut out_parameters = Vec::new();
    let mut out_parameters_names = Vec::new();
    'fn_loop: for capture in re.captures_iter(&filedata) {
        let (return_type, fn_name, param_string) = (
            capture.get(1).unwrap().as_str(),
            capture.get(2).unwrap().as_str(),
            capture.get(3).unwrap().as_str()
        );

        if return_type.ends_with("*") || param_string.contains("*") {  // we don't want pointers
            continue;
        }

        out_parameters.clear();
        out_parameters_names.clear();
        for parameter in param_string.split(",") {
            parameter_parts = parameter.split_ascii_whitespace().collect();

            if parameter_parts.len() < 2 {  // Ignore void declarations or declarations without a type.
                continue 'fn_loop;
            }

            parameter_end = parameter_parts[parameter_parts.len() -1];
            if parameter_end.ends_with("]") {  // is an array, thus a pointer from rust's level
                /* Find the length part of the array */
                position_start = parameter_end.chars().position(|c| c == '[').unwrap();
                position_end = parameter_end.len() - 1;
                parameter_arr_len = &parameter_end[position_start + 1 .. position_end];

                /* Parameter name */
                parameter_end = &parameter_end[..position_start];

                /* Obtain the reference operator and make the parameter call either .as_ptr() or .as_mut_ptr() */
                (mutability, param_type) = if parameter_parts[0] == "const" {
                    out_parameters_names.push(format!("{parameter_end}.as_ptr()"));
                    ("&", parameter_parts[1])  // index 1 has the type
                }
                else {
                    out_parameters_names.push(format!("{parameter_end}.as_mut_ptr()"));
                    ("&mut ", parameter_parts[0])  // index 0 has the type
                };

                if param_type.starts_with("mj") {
                    out_parameters.push(format!("{}: {mutability}[{}; {parameter_arr_len}]", parameter_end, capitalize(param_type)));
                }
                else {
                    out_parameters.push(format!("{}: {mutability}[std::ffi::c_{}; {parameter_arr_len}]", parameter_end, param_type));
                }
            }
            else {
                param_type = parameter_parts[0];
                if param_type.starts_with("mj") {
                    out_parameters.push(format!("{}: {}", parameter_parts[1], capitalize(param_type)));
                }
                else {
                    out_parameters.push(format!("{}: std::ffi::c_{}", parameter_parts[1], param_type));
                }
                out_parameters_names.push(parameter_parts[1].to_string());
            }
        }

        parameters_joined = out_parameters.join(", ");
        if return_type == "void" {
            return_type_out = String::new();
        }
        else if return_type.starts_with("mjt") {
            return_type_out = format!(" -> {}", capitalize(return_type));
        }
        else {
            return_type_out = format!(" -> std::ffi::c_{return_type}");
        };

        println!("
        pub fn {fn_name}({parameters_joined}){return_type_out}  {{
            unsafe {{ {fn_name}({}) }}
        }}
        ", out_parameters_names.join(", "));
    }
}
