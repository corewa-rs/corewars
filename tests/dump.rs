extern crate corewa_rs;

fn run_test(input: &str, expected_output: &str) {
    eprintln!("Parsing warrior:\n{}", input);

    let core = corewa_rs::parse(input).expect("Failed to parse test_file.red");

    eprintln!("Loaded core:\n{:?}", core);

    assert_eq!(core.dump(), expected_output);
}

#[test]
fn dump_all_opcodes() {
    run_test(
        include_str!("data/test.red"),
        include_str!("data/test_loadfile.red"),
    );
}

#[test]
#[ignore] // Until we have support for keeping metadata comments, EQU, etc.
fn dump_icws94_example_dwarf() {
    run_test(
        include_str!("data/dwarf.red"),
        include_str!("data/dwarf_loadfile.red"),
    );
}
