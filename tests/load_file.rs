extern crate corewa_rs;

use corewa_rs::load_file;

const TEST_PROGRAM: &'static str = "
    MOV.I 0, 1
    DAT 0, 0
";

#[test]
fn test_parse() {
    println!("Parsing program:\n{}", TEST_PROGRAM);

    let program = load_file::parse(TEST_PROGRAM);

    assert_eq!(program, load_file::Opcode::Mov);

    println!("{:?}", program);
}
