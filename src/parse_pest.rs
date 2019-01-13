use crate::load_file::{AddressMode, Field, Instruction, Opcode, Program, CORE_SIZE};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};

use std::str::FromStr;

#[derive(Parser)]
#[grammar = "redcode.pest"]
pub struct RedcodeParser;

pub fn parse(file_contents: &str) -> Program {
    let mut program = Program {
        instructions: vec![Instruction::default(); CORE_SIZE],
    };

    let parse_result = RedcodeParser::parse(Rule::assembly_file, file_contents)
        .expect("Error during parse of file")
        .next()
        .unwrap();

    for (i, instruction) in parse_result.into_inner().enumerate() {
        let instruction_inner = instruction.into_inner();
        if instruction_inner.peek().is_some() {
            program.instructions[i] = parse_instruction(instruction_inner);
        }
    }

    program
}

fn parse_instruction(mut instruction_pairs: Pairs<Rule>) -> Instruction {
    let opcode = parse_opcode(&instruction_pairs.next().unwrap());

    let a = parse_field(instruction_pairs.next().unwrap());

    let b = {
        if let Some(b_pair) = instruction_pairs.next() {
            Some(parse_field(b_pair))
        } else {
            None
        }
    };

    Instruction { opcode, a, b }
}

fn parse_opcode(opcode_pair: &Pair<Rule>) -> Opcode {
    Opcode::from_str(opcode_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_field(field_pair: Pair<Rule>) -> Field {
    println!("Parsing field: {:#?}", field_pair);

    let mut address_mode = AddressMode::default();
    let mut value = 0i32;

    for inner_pair in field_pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::mode => address_mode = AddressMode::from_str(inner_pair.as_str()).unwrap(),
            Rule::expr => value = parse_value(&inner_pair),
            unknown => panic!("Unexpected rule found: {:?}", unknown),
        }
    }

    Field {
        value,
        address_mode,
    }
}

fn parse_value(value_pair: &Pair<Rule>) -> i32 {
    i32::from_str_radix(value_pair.as_str(), 10).unwrap()
}
