extern crate corewa_rs;

use corewa_rs::load_file;

const TEST_PROGRAM: &str = "
    MOV.I 0, 1
    DAT 0, 0
";

#[test]
fn test_parse() {
    println!("Parsing program:\n{}", TEST_PROGRAM);

    let program = load_file::parse(TEST_PROGRAM);

    let first_opcode = program.get_opcode(0).unwrap();

    assert_eq!(first_opcode, load_file::Opcode::Mov);

    println!("{:#?}", program);
}
