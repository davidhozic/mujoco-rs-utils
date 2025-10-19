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


fn display_parsed(datatype: &str, name: &str, comment: &str, size_var: &str, size_mul: &str, accessor_prefix: &str) {
    // Convert to pascal case whenever the type starts with 'mj' or try to map to a Rust type.
    // If conversion fails, use the C FFI type.
    let datatype = if datatype.starts_with("mj") {datatype.to_pascal_case()} else {
        if let Some(rust_type) = C_TO_RUST_TYPE_MAPPING.get(datatype) {
            rust_type.to_string()
        } else { format!("std::ffi::c_{datatype}") }
    };

    // Create an array type if size is larger than one, otherwise assume scalar
    if size_mul == "1" || size_mul.len() == 0 {
        println!(
            "{name}: &[{datatype}{}; \"{}\"; {accessor_prefix}.{size_var}],",
            if datatype.starts_with("Mjt") && datatype != "MjtNum" && datatype != "MjtByte" {" [cast]"} else {""},
            comment.trim()
        );
    }
    else {
        println!("{name}: &[[{datatype}; {size_mul} as usize] [cast]; \"{}\"; {accessor_prefix}.{size_var}],", comment.trim());
    }
}

/// Try to extract an enum type from the documentation string.
/// E.g., dynamics type (mjtDyn) gets converted to ("dynamics type", "mjtDyn").
fn extract_possible_enum<'a>(docstring: &'a str, datatype: &'a str) -> (&'a str, &'a str) {
    if let Some(m) = ENUM_REGEX.captures(docstring) {
        let (_, [comment, enum_type]) = m.extract();
        (comment, enum_type)
    }
    else {
        (docstring, datatype)
    }
}


pub fn create_array_slice(structs_filepath: &Path, accessor_prefix: &str, struct_name: &str) {
    let filedata = fs::read_to_string(structs_filepath).unwrap();
    // Match the correct struct
    let re = regex::Regex::new(&format!(r"(?s)struct\s+{struct_name}\s*\{{.*?\n}};")).unwrap();
    let struct_data = re.find(&filedata).expect("failed to find struct or struct body").as_str();

    // Match the sizes that are marked with n something x some number
    let re = regex::Regex::new(r"(?m)((?:unsigned\s+)?[^\s]+)(?:\*\s+([^\s]+)|\s+([^\s]+)\[.+\]);\s+//(.*)\(([A-z]+)\s*(?:x|\*)\s*(\w+)\)$").unwrap();
    for capture in re.captures_iter(struct_data) {
        let (_, [mut datatype, name, mut comment, size_var, size_mul]) = capture.extract();
        (comment, datatype) = extract_possible_enum(comment, datatype);
        display_parsed(datatype, name, comment, size_var, size_mul, accessor_prefix);
    }

    // Match the sizes that are marked with some number * n something
    let re = regex::Regex::new(r"(?m)((?:unsigned\s+)?[^\s]+)(?:\*\s+([^\s]+)|\s+([^\s]+)\[.+\]);\s+//(.*)\(([0-9]*)\s*(?:\*|x)\s*(\w+)\)$").unwrap();
    for capture in re.captures_iter(struct_data) {
        let (_, [mut datatype, name, mut comment, size_mul, size_var]) = capture.extract();
        (comment, datatype) = extract_possible_enum(comment, datatype);
        display_parsed(datatype, name, comment, size_var, size_mul, accessor_prefix);
    }

    // Match the sizes that are marked with some fixed attribute for length
    let re = regex::Regex::new(r"(?m)((?:unsigned\s+)?[^\s]+)(?:\*\s+([^\s]+));\s+//(.*)\((\w+)\)$").unwrap();
    for capture in re.captures_iter(struct_data) {
        let (_, [mut datatype, name, mut comment, size_var]) = capture.extract();
        (comment, datatype) = extract_possible_enum(comment, datatype);
        display_parsed(datatype, name, comment, size_var, "", accessor_prefix);
    }
}
