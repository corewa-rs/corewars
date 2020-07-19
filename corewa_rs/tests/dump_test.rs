extern crate assert_that;
extern crate predicates;
extern crate test_case;
extern crate test_generator;

extern crate corewa_rs;

use std::fs;

use assert_that::assert_that;
use test_generator::test_resources;

#[test_resources("corewa_rs/tests/data/input/simple/*.red")]
// Commented out until expression parsing works properly
// #[test_resources("corewa_rs/tests/data/input/wilkie/*.red")]
// #[test_resources("corewa_rs/tests/data/input/willmoo/*.red")]
fn read_dir(input_file: &str) {
    // Workaround for the fact that `test_resources` paths are based on workspace Cargo.toml
    let current_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(current_dir.parent().unwrap()).unwrap();

    let input = fs::read_to_string(input_file)
        .unwrap_or_else(|err| panic!("Unable to read file {:?}: {:?}", input_file, err));

    let expected_out_file = input_file.replace("input", "expected_output");
    let expected_output = fs::read_to_string(expected_out_file)
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|err| panic!("Unable to read file {:?}: {:?}", input_file, err));

    let parsed_core = corewa_rs::parse(&input).unwrap_or_else(|e| panic!("Parse error:\n{}", e));

    assert_that!(
        &parsed_core.to_string().trim(),
        str::similar(expected_output)
    );
}
