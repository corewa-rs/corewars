use super::*;

#[test]
fn test_opcode_from_str()  {
    assert_eq!(Opcode::from_str("MOV").unwrap(), Opcode::Mov);
}

