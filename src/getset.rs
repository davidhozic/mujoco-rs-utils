//! Generator of get/set macros (impl_getter_setter)

use std::{collections::HashMap, path::Path};
use regex as re;
use std::fs;

/// Left part of the struct-matching regex pattern.
const RE_STRUCT_CONTENT_LEFT: &str = r"(?s)struct\s*";
/// Right part of the struct-matching regex pattern.
const RE_STRUCT_CONTENT_RIGHT: &str = r"+\s*\{.*?\}";


pub fn create_impl_getter_setter(cpath: &Path, struct_name: &str, accessor: &str) {
    let c_type_to_rust_type: HashMap<&str, &str> = HashMap::from_iter(
        [
            ("int", "u32"),
            ("uint64_t", "u64"),
            ("size_t", "usize"),
        ]
    );

    let struct_content_format = format!("{}{struct_name}{}", RE_STRUCT_CONTENT_LEFT, RE_STRUCT_CONTENT_RIGHT);
    let fdata = fs::read_to_string(cpath).unwrap();
    let re = re::Regex::new(&struct_content_format).unwrap();
    let struct_content = re.find(&fdata).unwrap().as_str();
    for line in struct_content.lines() {
        let splitw = line.split_ascii_whitespace();
        let mut splitw_iter = splitw.into_iter();

        // Match the type of continue if the line is not an attribute line
        let type_ = match splitw_iter.next() {
            Some(type_) => type_,
            None => continue
        };

        // Pointer types shouldn't have getters/setters.
        if type_.ends_with("*") {
            continue;
        }

        // Try to match the name if the lin is a valid attribute line
        let name = match splitw_iter.next() {
            Some(name) => {
                // Remove any [] part of the name
                if let Some(name_no_bracket) = name.split_once("[") {
                    name_no_bracket.0
                } else {
                    name.strip_suffix(";").unwrap_or(name)
                }
            }
            None => continue
        };

        // Try to match the "//" string to test if the line has a comment.
        match splitw_iter.next() {
            Some(c) => {
                if c != "//" {
                    continue;
                }
                c
            },
            None => continue
        };

        let s: Vec<&str> = splitw_iter.collect();
        let rest = s.join(" ");  // merge the rest (comment), with added space in between

        // println!("{type_} {name} {comment_start} {rest}");
        let rtype = if let Some(rtype) = c_type_to_rust_type.get(type_) {
            rtype.to_string()
        } else {
            if type_.starts_with("mj") {  // assume we have a type alias with correct PascalCase style.
                let mut chars = type_.chars();
                chars.next().unwrap().to_uppercase().to_string() + chars.as_str()  // capitalize the first letter
            }
            else {
                println!("warning: could not map C type {type_} to correct Rust type.");
                continue;
            }
        };

        println!(
            "impl_getter_setter!(get, {name}, \"{rest}\", {rtype}, {accessor});"
        );
    }
}
