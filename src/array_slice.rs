use std::path::Path;
use std::fs;
use inflector::Inflector;
use regex;


pub fn create_array_slice(structs_filepath: &Path, accessor_prefix: &str, struct_name: &str) {
    let filedata = fs::read_to_string(structs_filepath).unwrap();
    // Match the correct struct
    let re = regex::Regex::new(&format!(r"(?s)struct\s+{struct_name}\s*\{{.*?\n}};")).unwrap();
    let struct_data = re.find(&filedata).expect("failed to find struct or struct body").as_str();

    // Match the 
    let re = regex::Regex::new(r"([^\s]+)\*\s+([^\s]+);\s+//(.*)\(([A-z]+)\s*x\s*([0-9]+)\)").unwrap();
    for capture in re.captures_iter(struct_data) {
        let (_, [datatype, name, comment, size_var, size_mul]) = capture.extract();
        let datatype = if datatype.starts_with("mj") {datatype.to_pascal_case()} else {format!("std::ffi::c_{datatype}")};
        if size_mul == "1" {
            println!("{name}: &[{datatype}; \"{}\"; {accessor_prefix}.{size_var}],", comment.trim());
        }
        else {
            println!("{name}: &[[{datatype}; {size_mul}]; \"{}\"; {accessor_prefix}.{size_var}],", comment.trim());
        }
    }
}
