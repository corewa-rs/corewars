extern crate corewa_rs;
extern crate indoc;

use corewa_rs::load_file;
use indoc::indoc;

#[test]
fn test_parse() {
    let test_program = indoc!(r#"
        MOV 0, 1
        DAT 1, 2
        MOV 1, 0
    "#);

    println!("Parsing program:\n{}", test_program);

    let program = load_file::parse(test_program);

    println!("Loaded program:\n{}", program.dump());
}
