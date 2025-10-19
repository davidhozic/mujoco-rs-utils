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


fn display_parsed(datatype: &str, name: &str, comment: &str, size_var: &str, size_mul: &str, accessor_prefix: &str, summed_type: bool) {
    // Convert to pascal case whenever the type starts with 'mj' or try to map to a Rust type.
    // If conversion fails, use the C FFI type.
    let datatype = if datatype.starts_with("mj") {datatype.to_pascal_case()} else {
        if let Some(rust_type) = C_TO_RUST_TYPE_MAPPING.get(datatype) {
            rust_type.to_string()
        } else { format!("std::ffi::c_{datatype}") }
    };

    // A special case where the length of an array is a sum of values in some other array
    if summed_type {
        println!("{name}: &[{datatype}; \"{comment}\"; [{size_mul}; ({accessor_prefix}.{size_var}); ({accessor_prefix}.)]],");
    }

    // Create an array type if size is larger than one, otherwise assume scalar
    else if size_mul == "1" || size_mul.len() == 0 {
        println!(
            "{name}: &[{datatype}{}; \"{comment}\"; {accessor_prefix}.{size_var}],",
            if datatype.starts_with("Mjt") && datatype != "MjtNum" && datatype != "MjtByte" {" [cast]"} else {""}
        );
    }
    else {
        let type_cast = if size_mul.chars().all(char::is_numeric) {
            ""
        } else {" as usize"};
        println!("{name}: &[[{datatype}; {size_mul}{type_cast}] [cast]; \"{comment}\"; {accessor_prefix}.{size_var}],");
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

    println!("------------------------------------------------");
    println!("Processing lengths obtained via single attribute");
    println!("------------------------------------------------");

    // Match the sizes that are marked with n something x some number
    let re = regex::Regex::new(r"(?m)((?:unsigned\s+)?[^\s]+)(?:\*\s+([^\s]+)|\s+([^\s]+)\[.+\]);\s+//(.*)\(([A-z]+)\s*(?:x|\*)\s*(\w+)\)$").unwrap();
    for capture in re.captures_iter(struct_data) {
        let (_, [mut datatype, name, mut comment, size_var, size_mul]) = capture.extract();
        (comment, datatype) = extract_possible_enum(comment, datatype);
        display_parsed(datatype, name, comment.trim(), size_var, size_mul, accessor_prefix, false);
    }

    // Match the sizes that are marked with some number * n something
    let re = regex::Regex::new(r"(?m)((?:unsigned\s+)?[^\s]+)(?:\*\s+([^\s]+)|\s+([^\s]+)\[.+\]);\s+//(.*)\(([0-9]*)\s*(?:\*|x)\s*(\w+)\)$").unwrap();
    for capture in re.captures_iter(struct_data) {
        let (_, [mut datatype, name, mut comment, size_mul, size_var]) = capture.extract();
        (comment, datatype) = extract_possible_enum(comment, datatype);
        display_parsed(datatype, name, comment.trim(), size_var, size_mul, accessor_prefix, false);
    }

    // Match the sizes that are marked with some fixed attribute for length
    let re = regex::Regex::new(r"(?m)((?:unsigned\s+)?[^\s]+)(?:\*\s+([^\s]+)|\s+([^\s]+)\[\w*\]);\s+//(.*)\((\w+)\)$").unwrap();
    for capture in re.captures_iter(struct_data) {
        let (_, [mut datatype, name, mut comment, size_var]) = capture.extract();
        (comment, datatype) = extract_possible_enum(comment, datatype);
        display_parsed(datatype, name, comment.trim(), size_var, "", accessor_prefix, false);
    }

    println!("--------------------------------------------------------");
    println!("Processing lengths obtained via sum of some length array");
    println!("--------------------------------------------------------");

    // Match summed length array
    let re = regex::Regex::new(r"(?m)((?:unsigned\s+)?[^\s]+)(?:\*\s+([^\s]+));\s+//(.*)\(\s*([0-9])+\s*[*x]\s*sum\((\w+)\)\s*\)$").unwrap();
    for capture in re.captures_iter(struct_data) {
        let (_, [mut datatype, name, mut comment, size_mul, size_var]) = capture.extract();
        (comment, datatype) = extract_possible_enum(comment, datatype);
        display_parsed(datatype, name, comment.trim(), size_var, size_mul, accessor_prefix, true);
    }
}
