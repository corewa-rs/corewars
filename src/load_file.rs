use std::{convert::AsRef, fmt::Debug, vec};

use nom::IResult;

#[derive(Debug)]
pub struct Program {
    bytes: vec::Vec<u8>,
}

impl Program {}

#[derive(Debug, PartialEq)]
pub enum Opcode {
    Mov,
    Dat,
}

named!(parse_opcode<&str, &str>,
    ws!(alt!(tag_s!("MOV") | tag_s!("DAT")))
);

fn get_opcode(result: IResult<&str, &str>) -> Opcode {
    match result {
        Ok((rest, value)) => match value {
            "MOV" => Opcode::Mov,
            "DAT" => Opcode::Dat,
            _ => panic!("Unexpected opcode"),
        },
        Err(_) => {
            panic!("Error parsing");
        }
    }
}

pub fn parse(file_contents: &str) -> Opcode {
    let parse_result = parse_opcode(file_contents);

    get_opcode(parse_result)
}
