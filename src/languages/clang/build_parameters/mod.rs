use rust_macro_internal::*;
use parser_lib::json::*;

json_struct!("src/languages/clang/build_parameters/parameter.json", CLangBuildParameters);

impl Default for CLangBuildParameters {
    fn default() -> Self {
        let json_str = include_str!("parameter.json");
        CLangBuildParameters::from_json_string(json_str)
    }
}

impl CLangBuildParameters {
    pub fn new_default_test_params() -> Self {
        let json_str = include_str!("parameter_single_test.json");
        CLangBuildParameters::from_json_string(json_str)
    }

    pub fn new_default_batch_test_params() -> Self {
        let json_str = include_str!("parameter_single_batch_test.json");
        CLangBuildParameters::from_json_string(json_str)
    }
}