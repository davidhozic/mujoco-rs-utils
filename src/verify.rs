//! Verification module. Code in this module is used.
//! to verify if selected items have been correctly
//! defined and used.
use regex;



/// Matches invocations of info_method! and extract their parameter information.
const RE_INFO_METHOD_MATCH: &str = r"info_method!\s*\{\s*(?<struct_type>\w+)\s*,\s*(?<ffi>.+?)\s*,\s*(?<view_of>\w+)\s*,\s*\[\s*(?<fixed_size_fields>[\w:\s*,]*)\s*\]\s*,\s*\[\s*(?<ffi_obtained_size_fields>[\w:\s*,]*)\s*\]\s*,\s*\[\s*(?<instance_dependent_size_fields>[\w:\s*,]*)\s*\]\s*\}";


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_method_match() {
        const TEST_INFO_METHOD_INVOKE: &str = "
        info_method! { Model, ffi(), body,
            [parentid: 1, rootid: 2, weldid: 3, mocapid: 4],
            [user: nuser_site * 5],
            [objtype: ntupledata, objid: ntupledata]
        }";

        let re = regex::Regex::new(RE_INFO_METHOD_MATCH).unwrap();
        for capture in re.captures_iter(TEST_INFO_METHOD_INVOKE) {
            let struct_type = capture.name("struct_type").map_or("", |x| x.as_str());
            let ffi = capture.name("ffi").map_or("", |x| x.as_str());
            let view_of = capture.name("view_of").map_or("", |x| x.as_str());
            let fixed_size_fields = capture.name("fixed_size_fields").map_or("", |x| x.as_str());
            let ffi_obtained_size_fields = capture.name("ffi_obtained_size_fields").map_or("", |x| x.as_str());
            let instance_dependent_size_fields = capture.name("instance_dependent_size_fields").map_or("", |x| x.as_str());

            assert_eq!(struct_type, "Model");
            assert_eq!(view_of, "body");
            assert_eq!(ffi, "ffi()");
            assert_eq!(fixed_size_fields, "parentid: 1, rootid: 2, weldid: 3, mocapid: 4");
            assert_eq!(ffi_obtained_size_fields, "user: nuser_site * 5");
            assert_eq!(instance_dependent_size_fields, "objtype: ntupledata, objid: ntupledata");
        }
    }
}
