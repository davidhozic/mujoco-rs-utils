use std::cell::LazyCell;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use inflector::Inflector;
use regex::{self, Regex};


const C_TO_RUST_TYPE_MAPPING: LazyCell<HashMap<&'static str, &'static str>> = LazyCell::new(
    || HashMap::from_iter([
        ("float", "f32"),
        ("double", "f64"),
        ("char", "i8"),
        ("int", "i32"),
        ("unsigned char", "u8"),
        ("unsigned int", "u32"),
    ])
);

const ENUM_REGEX: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"(.*?)\((mjt[A-z]+)\)").unwrap());
const ATTR_NORMAL_REGEX: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"(?m)((?:unsigned\s+)?[^\s*]+)(?:\s+([^\s\[]+));\s+//\s*(.*)$").unwrap());
const ATTR_ARRAY_REGEX: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"(?m)((?:unsigned\s+)?[^\s*]+)(?:\s+([^\s]+)\[(.+)\]);\s+//\s*(.*)$").unwrap());


/// Try to extract an enum type from the documentation string.
/// E.g., dynamics type (mjtDyn) gets converted to ("dynamics type", "mjtDyn").
fn extract_possible_enum<'a>(docstring: &'a str, datatype: &'a str) -> (&'a str, &'a str, bool) {
    if let Some(m) = ENUM_REGEX.captures(docstring) {
        let (_, [comment, enum_type]) = m.extract();
        (comment, enum_type, true)
    }
    else {
        (docstring, datatype, false)
    }
}


/// Postprocess a datatype to correct Rust type
fn convert_type(datatype: &str) -> String {
    if datatype.starts_with("mj") {datatype.to_pascal_case()} else {
        if let Some(rust_type) = C_TO_RUST_TYPE_MAPPING.get(datatype) {
            rust_type.to_string()
        } else { format!("std::ffi::c_{datatype}") }
    }
}


pub fn create_getters_setters(structs_filepath: &Path, struct_name: &str) {
    let filedata = fs::read_to_string(structs_filepath).unwrap();
    // Match the correct struct
    let re = regex::Regex::new(&format!(r"(?s)struct\s+{struct_name}\s*\{{.*?\n}};")).unwrap();
    let struct_data = re.find(&filedata).expect("failed to find struct or struct body").as_str();
    let mut was_enum;

    println!("-----------------------------------------");
    println!("Processing normal getters/setters/builders");
    println!("-----------------------------------------");
    for capture in ATTR_NORMAL_REGEX.captures_iter(struct_data) {
        let (_, [mut datatype, name, mut comment]) = capture.extract();
        (comment, datatype, was_enum) = extract_possible_enum(comment, datatype);
        let datatype_owned = convert_type(datatype);

        if datatype_owned.starts_with("Mj") && !datatype.starts_with("Mjt") {  // complex type, only allow references
            print!("{name}: &{datatype_owned}; \"{}.\";", comment.trim())
        }
        else {  // Create a scalar getter/setter/builder
            print!("{name}: {datatype_owned}; \"{}.\";", comment.trim())
        }

        if was_enum {
            print!(" // force type coercion!")
        }
        println!();
    }

    println!("-----------------------------------------");
    println!("Processing array getters/setters/builders");
    println!("-----------------------------------------");
    for capture in ATTR_ARRAY_REGEX.captures_iter(struct_data) {
        let (_, [mut datatype, name, size, mut comment]) = capture.extract();
        (comment, datatype, _) = extract_possible_enum(comment, datatype);
        let datatype_owned = convert_type(datatype);
        let type_cast = if size.chars().next().unwrap().is_alphabetic() { " as usize" } else {""};
        println!("{name}: &[{datatype_owned}; {size}{type_cast}]; \"{}.\";", comment.trim());
    }
}
