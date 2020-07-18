extern crate assert_that;
extern crate predicates;
extern crate test_case;

extern crate corewa_rs;

use assert_that::assert_that;
use test_case::test_case;

#[test_case(
    include_str!("data/basic.red"),
    include_str!("data/basic_loadfile.red");
    "simple test"
)]
#[test_case(
    include_str!("data/dwarf.red"),
    include_str!("data/dwarf_loadfile.red");
    "sample dwarf warrior"
)]
fn run_test(input: &str, expected_output: &'static str) {
    let parsed_core = corewa_rs::parse(input).unwrap_or_else(|e| panic!("Parse error:\n{}", e));

    assert_that!(
        &parsed_core.to_string().trim(),
        str::similar(expected_output.trim())
    );
}
