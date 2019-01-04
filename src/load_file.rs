use std::vec;

use nom::IResult;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Opcode {
    Mov,
    Dat,
}

#[derive(Debug)]
enum Increment {
    Post,
    Pre,
    None,
}

#[derive(Debug)]
enum AddressMode {
    Immediate,
    Direct,
    IndirectA(Increment),
    IndirectB(Increment),
}

#[derive(Debug)]
struct Field {
    value: i32,
    address_mode: AddressMode,
}

#[derive(Debug)]
struct Instruction {
    pub opcode: Opcode,
    a: Field,
    b: Field,
}

#[derive(Debug)]
pub struct Program {
    instructions: vec::Vec<Instruction>,
}

impl Program {
    pub fn get_opcode(&self, index: usize) -> Option<Opcode> {
        match self.instructions.get(index) {
            Some(instruction) => Some(instruction.opcode),
            _ => None,
        }
    }
}

named!(parse_opcode<&str, &str>,
    ws!(alt!(tag_s!("MOV") | tag_s!("DAT")))
);

fn get_opcode(result: IResult<&str, &str>) -> Opcode {
    match result {
        Ok((_rest, value)) => match value {
            "MOV" => Opcode::Mov,
            "DAT" => Opcode::Dat,
            _ => panic!("Unexpected opcode"),
        },
        Err(_) => {
            panic!("Error parsing");
        }
    }
}

pub fn parse(file_contents: &str) -> Program {
    let parse_result = parse_opcode(file_contents);

    Program {
        instructions: vec![Instruction {
            opcode: get_opcode(parse_result),
            a: Field {
                value: 0,
                address_mode: AddressMode::Direct,
            },
            b: Field {
                value: 0,
                address_mode: AddressMode::Direct,
            },
        }],
    }
}
