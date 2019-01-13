extern crate corewa_rs;

use corewa_rs::load_file;

#[test]
fn test_parse() {
    let test_program = include_str!("test_file.red");

    println!("Parsing program:\n{}", test_program);

    let program = load_file::parse(test_program);

    println!("Loaded program:\n{}", program.dump());
}
