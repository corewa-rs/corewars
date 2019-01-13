use std::{str::FromStr, vec};

use nom::{digit, IResult};

use crate::load_file::{Field, Instruction, Opcode, Program, CORE_SIZE};

named!(take_opcode<&str, Opcode>, ws!(
    map_res!(
        alt!(
            tag!("MOV") | tag!("DAT")
        ),
        FromStr::from_str
    ))
);

named!(int32<&str, i32>, ws!(
    map_res!(digit, FromStr::from_str))
);

named!(take_instruction<&str, Instruction>, ws!(
    do_parse!(
        op: take_opcode >>
        field_a: int32 >> tag!(",") >>
        field_b: int32 >>
        (Instruction {
            opcode: op,
            a: Field{value: field_a, ..Default::default()},
            b: Some(Field{value: field_b, ..Default::default()}),
        })
    )
));

named!(take_program<&str, vec::Vec<Instruction>>, many0!(take_instruction));

pub fn parse(file_contents: &str) -> Program {
    let mut program = Program {
        instructions: vec![Instruction::default(); CORE_SIZE],
    };

    // TODO proper error handling
    if let IResult::Done(_, parsed_program) = take_program(file_contents) {
        for (i, &instruction) in parsed_program.iter().enumerate() {
            program.instructions[i] = instruction;
        }
    } else {
        println!("Parser was not Done")
    }

    program
}
