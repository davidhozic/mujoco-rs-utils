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

pub fn create_info_calls(filepath: &Path) {
    /* Read the contents of the file containing the view defines (indexer_xmacro.h) */
    let filedata = fs::read_to_string(filepath).expect("could not read the file");

    /* Create regex patterns */
    // Match the entire define block
    let re = regex::RegexBuilder::new(RE_DEFINE_PATTERN)
        .dot_matches_new_line(true)
        .multi_line(true)
        .build().expect("invalid regex");

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
        let mut fixed_length_attributes = vec![];
        let mut external_length_attributes = vec![];
        let mut dynamic_length_attributes = vec![];
        /* Parse individual X(..) */
        for line in capture_data.lines().skip(1) {  // skip the #define line
            // Remove parentheses, trim whitespace and parse into parts.
            let mut parts: Vec<_> = line.split(",").map(|item| item.trim()).collect();
            *parts.first_mut().unwrap() = parts.first().unwrap().split_once("(").unwrap().1;

            // Split at last ). Multiple ) can appear due to MJ_M(...).
            *parts.last_mut().unwrap() = parts.last().unwrap().rsplit_once(")").unwrap().0.trim();

            if let [_, _, suffix, ntotaldim, dim] = &mut parts[..] {
                // Match the number of dimensions string to correct mapping address array in MjModel
                if dim.starts_with("MJ_M(") {
                    let (left, mut right) = dim.strip_prefix("MJ_M(").unwrap().split_once(")").unwrap();
                    *dim = left;
                    right = right.trim();
                    external_length_attributes.push(format!("{suffix}: {left}{right}"));
                }
                else if ntotaldim.len() > 2 || NX_ALLOWED_DIRECT_LENGTH.contains(ntotaldim) {
                    fixed_length_attributes.push(format!("{suffix}: {dim}"));
                }
                else {
                    dynamic_length_attributes.push(format!("{suffix}: {ntotaldim}"));
                }
            }
        }

        if fixed_length_attributes.len() + dynamic_length_attributes.len() + external_length_attributes.len() > 0 {
            println!(
                "info_method! {{ {}, ffi(), {},\
                    \n\t       [{}],\
                    \n\t       [{}],\
                    \n\t       [{}] }}\n",
                class.to_pascal_case(), item.to_lowercase(),
                fixed_length_attributes.join(", "),
                external_length_attributes.join(", "),
                dynamic_length_attributes.join(", "),
            );
        }
    }
}
