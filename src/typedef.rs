//! Module for generating type definitions of existing types, to match PascalCase.

use std::{fs, path::Path};
use inflector::Inflector;
use regex;

pub fn create_types(api_reference: &Path, prefix: Option<&str>) {
    let filedata = fs::read_to_string(api_reference).unwrap();
    let struct_pat = prefix.unwrap_or("");
    let re = regex::Regex::new(&format!(r"(\w+[\w\s`:<> \->.]*?)\.\.\s+mujoco-include::\s*({struct_pat}[A-z]+)")).unwrap();
    for captures in re.captures_iter(&filedata) {
        let docstring = captures.get(1).unwrap().as_str().trim();

        let name = captures.get(2).unwrap().as_str().trim_end_matches("_");
        for line in docstring.lines() {
            println!("/// {line}");
        }
        println!("type {} = {};\n", name.to_pascal_case(), name);
    }
}
