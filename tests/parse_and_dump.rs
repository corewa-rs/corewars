extern crate corewa_rs;

use corewa_rs::parser;

#[test]
fn test_parse() {
    let test_warrior = include_str!("test_file.red");

    println!("Parsing warrior:\n{}", test_warrior);

    let core = parser::parse(test_warrior).unwrap();

    println!("Loaded core:\n{:?}", core);

    let expected_core_dump = include_str!("expected_loadfile.red");

    assert_eq!(core.dump(), expected_core_dump);
}
