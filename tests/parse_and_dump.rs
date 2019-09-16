extern crate corewa_rs;

#[test]
fn test_parse() {
    let test_warrior = include_str!("data/test_file.red");

    println!("Parsing warrior:\n{}", test_warrior);

    let core = corewa_rs::parse(test_warrior).unwrap();

    println!("Loaded core:\n{:?}", core);

    let expected_core_dump = include_str!("data/expected_loadfile.red");

    assert_eq!(core.dump(), expected_core_dump);
}
