extern crate corewa_rs;

fn run_test(input: &str, expected_output: &'static str) {
    let mut parsed_core = corewa_rs::parse(input).unwrap_or_else(|e| panic!("Parse error:\n{}", e));

    // TODO: dump and check output of pre-resolved core
    parsed_core
        .result
        .resolve()
        .unwrap_or_else(|e| panic!("{}", e));

    testutils::assert_that!(
        &parsed_core.to_string(),
        predicates::str::similar(expected_output),
    );
}

#[test]
fn dump_all_opcodes() {
    run_test(
        include_str!("data/test.red"),
        include_str!("data/test_loadfile.red"),
    );
}

#[test]
#[ignore = "Fails for metadata comments, EQU, etc."]
fn dump_icws94_example_dwarf() {
    run_test(
        include_str!("data/dwarf.red"),
        include_str!("data/dwarf_loadfile.red"),
    );
}
