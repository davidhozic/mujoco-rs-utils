//! Module for generating info method code to MjModel and MjData.
use std::path::Path;
use std::fs;

use inflector::Inflector;
use regex;

const RE_DEFINE_PATTERN: &str = r"#define MJ(?<class>[A-z]+)_(?<item>[A-z]+).*?\)$";
/// Allowed total array dimension names to consider for direct mapping. Names that are outside
/// of this array will only be directly mapped if their length is greater than 2, as we
/// assume those correspond to the item length and not the total array length.
/// We assume that nx means length of an array, whose elements are not of fixed-length.
const NX_ALLOWED_DIRECT_LENGTH: [&str; 1] = ["nu"];

pub fn create_views(filepath: &Path) {
    /* Read the contents of the file containing the view defines (indexer_xmacro.h) */
    let filedata = fs::read_to_string(filepath).expect("could not read the file");

    /* Create regex patterns */
    // Match the entire define block
    let re = regex::RegexBuilder::new(RE_DEFINE_PATTERN)
        .dot_matches_new_line(true)
        .multi_line(true)
        .build().expect("invalid regex");

    let mut info_method_calls = vec![];
    let mut info_with_view_calls = vec![];
    for capture in re.captures_iter(&filedata) {
        let (class, item, capture_data) = (
            capture.name("class").expect("could not find the class type").as_str().to_lowercase(),
            capture.name("item").expect("could not find the item type").as_str(),
            capture.get(0).unwrap().as_str()
        );

        //     info_method! { Model, ffi(), sensor, [
        //     r#type: 1, datatype: 1, needstage: 1,
        //     objtype: 1, objid: 1, reftype: 1, refid: 1, intprm: mjNSENS as usize,
        //     dim: 1, adr: 1, cutoff: 1, noise: 1
        // ], [], []}
        let mut fixed_length_attributes_lengths = vec![];
        let mut external_length_attributes_lengths = vec![];
        let mut dynamic_length_attributes_lengths = vec![];

        // info_with_view!(Data, actuator, [ctrl: MjtNum], [act: MjtNum], M: Deref<Target = MjModel>);
        let mut attribute_types_and_names = vec![];

        /* Parse individual X(..) */
        for line in capture_data.lines().skip(1) {  // skip the #define line
            // Remove parentheses, trim whitespace and parse into parts.
            let mut parts: Vec<_> = line.split(",").map(|item| item.trim()).collect();
            *parts.first_mut().unwrap() = parts.first().unwrap().split_once("(").unwrap().1.trim();

            // Split at last ). Multiple ) can appear due to MJ_M(...).
            *parts.last_mut().unwrap() = parts.last().unwrap().rsplit_once(")").unwrap().0.trim();

            if let [type_, prefix, attribute, ntotaldim, dim] = &parts[..] {
                let type_ = match *type_ {
                    "float" => "f32".to_string(),
                    "double" => "f64".to_string(),
                    "int" => "i32".to_string(),
                    _ if type_.starts_with("mjt") => type_.to_pascal_case(),
                    _ => type_.to_string(),
                };

                // Some attributes may have _ added in front of them instead of at the prefix.
                // This ensures our view attributes don't add _ to the attribute name but instead add the _
                // to the prefix end.
                let (extra_prefix, attribute) = if attribute.starts_with("_") {
                    ("_", attribute.strip_prefix("_").unwrap())
                } else { ("", *attribute) };

                let prefix = prefix.trim();
                let prefix_str = if !prefix.is_empty() {
                    format!("[{prefix}{extra_prefix}] ")
                } else {
                    "".to_string()
                };

                let cast_str = if type_.starts_with("Mjt") &&
                    type_ != "MjtByte" &&
                    type_ != "MjtSize" &&
                    type_ != "MjtNum"
                {
                    " [cast]"
                } else {
                    ""
                };

                // Exceptions: reserved keywords cannot be attribute names
                let attribute_extra =  if attribute == "type" {
                     "r#"
                } else { "" };

                attribute_types_and_names.push(format!("{prefix_str}{attribute_extra}{attribute}: {type_}{cast_str}"));

                // Match the number of dimensions string to correct mapping address array in MjModel
                if dim.starts_with("MJ_M(") {
                    let (left, mut right) = dim.strip_prefix("MJ_M(").unwrap().split_once(")").unwrap();
                    right = right.trim();
                    external_length_attributes_lengths.push(format!("{attribute_extra}{attribute}: {left}{right}"));
                } else if ntotaldim.len() > 2 || NX_ALLOWED_DIRECT_LENGTH.contains(ntotaldim) {
                    fixed_length_attributes_lengths.push(format!("{attribute_extra}{attribute}: {dim}"));
                } else {
                    dynamic_length_attributes_lengths.push(format!("{attribute_extra}{attribute}: {ntotaldim}"));
                }
            }
        }

        if fixed_length_attributes_lengths.len() + dynamic_length_attributes_lengths.len() + external_length_attributes_lengths.len() > 0 {
            let class = class.to_pascal_case();
            let item = item.to_lowercase();
            info_method_calls.push(format!(
                "info_method! {{ {class}, ffi(), {item},\
                    \n\t[{}],\
                    \n\t[{}],\
                    \n\t[{}]\
                    \n}}",
                join_attributes_chunked_pretty(&fixed_length_attributes_lengths, 3),
                join_attributes_chunked_pretty(&external_length_attributes_lengths, 3),
                join_attributes_chunked_pretty(&dynamic_length_attributes_lengths, 3),
            ));

            // Generate info and view structs. Here we assume all attributes are mandatory
            // as there is no way to check this here (MANUAL CHECK REQUIRED!).
            info_with_view_calls.push(format!(
                "info_with_view!({class}, {item},\n\t[{}],\n\t[]{});",
                join_attributes_chunked_pretty(&attribute_types_and_names, 10),
                if class == "Data" {
                    ", M: Deref<Target = MjModel>"  // MjData has this trait bound.
                } else { "" }  // MjModel and others have no trait bound.
            ));
        }
    }

    for info_method_call in &info_method_calls {
        println!("{info_method_call}\n");
    }

    for info_with_view_call in &info_with_view_calls {
        println!("{info_with_view_call}\n");
    }
}

/// Joins chunks of text elements together in such way that it ends up
/// prettified. The `n_lines_target` specifies the approximate number of lines
/// to have in the end.
fn join_attributes_chunked_pretty(data: &[String], n_lines_target: usize) -> String {
    data.chunks((data.len() / n_lines_target).max(1)).map(|chunk| chunk.join(", "))
        .collect::<Vec<_>>()
        .join(",\n\t ")
}
