extern crate corewa_rs;

use corewa_rs::parse_nom;
use corewa_rs::parse_pest;

#[test]
fn test_parse_nom() {
    let test_program = include_str!("test_file.red");

    println!("Parsing program:\n{}", test_program);

    let program = parse_nom::parse(test_program);

    let program_dump = program.dump();
    println!("Loaded program:\n{}", program_dump);

    let expected_program_dump = include_str!("expected_loadfile.red");

    assert_eq!(program_dump, expected_program_dump);
}

#[test]
fn test_parse_pest() {
    let test_program = include_str!("test_file.red");

    println!("Parsing program:\n{}", test_program);

    let program = parse_pest::parse(test_program);

    let program_dump = program.dump();
    println!("Loaded program:\n{}", program_dump);

    let expected_program_dump = include_str!("expected_loadfile.red");

    assert_eq!(program_dump, expected_program_dump);
}
