//! Module for creating MjModel methods
use inflector::Inflector;
use std::path::Path;
use std::fs;
use regex;


const RE_SIGNATURE: &str = r"MJAPI\s+((?:const\s+)?[\w*]+)\s+(\w+)\s*\((.*mjModel.*)\)";


pub fn create_model_methods(path: &Path) {
    let re = regex::Regex::new(RE_SIGNATURE).unwrap();
    let fdata = fs::read_to_string(path).unwrap();

    for capture in re.captures_iter(&fdata) {
        let (return_type, fn_name, param_string) = (
            capture.get(1).unwrap().as_str(),
            capture.get(2).unwrap().as_str(),
            capture.get(3).unwrap().as_str(),
        );
        
        if param_string.contains("mjData") {
            continue;
        }

        if let Some((params, param_names)) = process_arguments(param_string) {
            // println!("{fn_name}({param_string}); Parsed: {params:?} {param_names:?}");
            println!("
            pub fn {}({}) {{
                unsafe {{ {fn_name}({}) }}
            }}
            ", fn_name.trim_start_matches("mj_").to_snake_case(),
            params.join(", "), param_names.join(", "));
        }
    }
}


fn process_arguments(param_string: &str) -> Option<(Vec<String>, Vec<String>)> {
    let mut out_parameters = Vec::new();
    let mut out_parameters_names = Vec::new();
    let mut parameter_parts: Vec<_>;
    let mut param_type;
    let mut param_name;
    let mut mutability;
    for parameter in param_string.split(",") {
        parameter_parts = parameter.split_ascii_whitespace().collect();

        let parameter_end = parameter_parts[parameter_parts.len() -1];
        if parameter_end.ends_with("]") {  // is an array, thus a pointer from rust's level
            /* Find the length part of the array */
            let position_start = parameter_end.chars().position(|c| c == '[').unwrap();
            let position_end = parameter_end.len() - 1;
            let parameter_arr_len = &parameter_end[position_start + 1 .. position_end];

            /* Parameter name */
            let parameter_name = parameter_end[..position_start].to_snake_case();

            /* Obtain the reference operator and make the parameter call either .as_ptr() or .as_mut_ptr() */
            (mutability, param_type) = if parameter_parts[0] == "const" {
                out_parameters_names.push(format!("{parameter_name}.as_ptr()"));
                ("&", parameter_parts[1])  // index 1 has the type
            }
            else {
                out_parameters_names.push(format!("{parameter_name}.as_mut_ptr()"));
                ("&mut ", parameter_parts[0])  // index 0 has the type
            };
            
            if param_type.starts_with("void") {
                return None;
            }

            if param_type.starts_with("mj") {
                out_parameters.push(format!("{}: {mutability}[{}; {parameter_arr_len}]", parameter_name, param_type.to_pascal_case()));
            }
            else {
                out_parameters.push(format!("{}: {mutability}[std::ffi::c_{}; {parameter_arr_len}]", parameter_name, param_type));
            }
        }
        else {
            if parameter_parts[0] == "const" && parameter_parts[1] == "mjModel*"  {
                out_parameters_names.push("self.ffi()".into());
                out_parameters.insert(0, "&self".into());
            }
            else if parameter_parts[0] == "mjModel*" {
                out_parameters_names.push("self.ffi_mut()".into());
                out_parameters.insert(0, "&mut self".into());
            }
            else {
                (param_type, mutability) = if parameter_parts[0] == "const" {
                    (parameter_parts[1], "&")
                }
                else {
                    (parameter_parts[0], "&mut ")
                };

                let is_pointer = param_type.ends_with("*");
                let mut param_type_string = if param_type.starts_with("mj") {
                    param_type.to_pascal_case()
                }
                else {
                    format!("std::ffi::c_{}", param_type.trim_end_matches("*"))
                };

                if is_pointer {
                    param_type_string = mutability.to_string() + &param_type_string;
                }

                param_name = parameter_parts[parameter_parts.len() - 1].to_snake_case();
                out_parameters.push(format!("{}: {}", param_name, param_type_string));
                out_parameters_names.push(param_name);
            }
        }
    }
    Some((out_parameters, out_parameters_names))
}
