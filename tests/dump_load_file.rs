extern crate corewa_rs;

use corewa_rs::parser;

#[test]
fn test_parse() {
    let test_warrior = include_str!("test_file.red");

    println!("Parsing program:\n{}", test_warrior);

    let core = parser::parse(test_warrior);

    let core_dump = core.dump();
    println!("Loaded program:\n{}", core_dump);

    let expected_core_dump = include_str!("expected_loadfile.red");

    assert_eq!(core_dump, expected_core_dump);
}
