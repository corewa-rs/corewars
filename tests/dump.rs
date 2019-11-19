extern crate corewa_rs;

fn run_test(input: &str, expected_output: &str) {
    eprintln!("Parsing warrior:");
    eprintln!("{}", input);

    let core = corewa_rs::parse(input).expect("Failed to parse input");

    eprintln!("Loaded core:");
    eprintln!("{:?}", core);

    assert_eq!(core.dump(), expected_output);
}

#[test]
#[ignore = "Fails because labels are not yet converted to offsets"]
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
