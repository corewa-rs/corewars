extern crate corewa_rs;

#[test]
fn test_parse() {
    let test_program = include_str!("test_file.red");

    println!("Parsing program:\n{}", test_program);

    let program = corewa_rs::parser::parse(test_program).unwrap();

    let program_dump = program.dump();
    println!("Loaded program:\n{}", program_dump);

    let expected_program_dump = include_str!("expected_loadfile.red");

    assert_eq!(program_dump, expected_program_dump);
}
