//! Module for generating views code to MjModel and MjData.
use std::path::Path;
use std::fs;


use regex;

const RE_DEFINE_PATTERN: &str = r"#define MJ(?<class>[A-z]+)_(?<item>[A-z]+).*?\)$";

pub fn create_views(filepath: &Path) {
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

        println!("{class}: {item}");
        /* Parse individual X(..) */
        for line in capture_data.lines().skip(1) {  // skip the #define line
            // Remove parentheses, trim whitespace and parse into parts.
            let parts: Vec<_> = line.split(",").map(|mut item| {
                item = item.split_once("(").map_or(item, |x| x.1);
                item = item.split_once(")").map_or(item, |x| x.0);
                item.trim()
            }).collect();

            if let [_, _, suffix, ntotaldim, dim] = &parts[..] {
                // Match the number of dimensions string to correct mapping address array in MjModel
                if *dim != "1" {
                    println!("      let {suffix} = (id * {dim}, {dim});");
                }
                else {
                    println!("      let {suffix} = mj_view_indices!(id, mj_model_nx_to_mapping!(model_ffi, {ntotaldim}), mj_model_nx_to_nitem!(model_ffi, {ntotaldim}), model_ffi.{ntotaldim});");
                }
            }
        }
    }
}
