use std::{convert::From, fmt, str::FromStr};

use pest::{
    error::Error as PestError,
    iterators::{Pair, Pairs},
    Parser,
};

use crate::load_file::{AddressMode, Core, Field, Instruction, Opcode};

pub struct Error {
    details: String,
}

impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.details)
    }
}

impl Error {
    pub fn no_input() -> Error {
        Error {
            details: "No input found".to_owned(),
        }
    }
}

impl<Rule> From<PestError<Rule>> for Error {
    fn from(pest_error: PestError<Rule>) -> Error {
        Error {
            details: format!(
                "Error parsing rule '{}' at location '{:?}",
                stringify!(Rule),
                pest_error.line_col
            ),
        }
    }
}

#[derive(Parser)]
#[grammar = "data/redcode.pest"]
struct RedcodeParser;

pub fn parse(file_contents: &str) -> Result<Core, Error> {
    if file_contents.is_empty() {
        return Err(Error::no_input());
    }

    let mut core = Core::default();

    let parse_result = RedcodeParser::parse(Rule::assembly_file, file_contents)?
        .next()
        .ok_or_else(Error::no_input)?;

    for (i, instruction) in parse_result.into_inner().enumerate() {
        let instruction_inner = instruction.into_inner();
        if instruction_inner.peek().is_some() {
            core.set(i, parse_instruction(instruction_inner));
        }
    }

    Ok(core)
}

fn parse_instruction(mut instruction_pairs: Pairs<Rule>) -> Instruction {
    let opcode = parse_opcode(&instruction_pairs.next().unwrap());

    let field_a = parse_field(instruction_pairs.next().unwrap());
    let field_b = instruction_pairs
        .next()
        .map_or_else(Field::default, parse_field);

    Instruction {
        opcode,
        field_a,
        field_b,
    }
}

fn parse_opcode(opcode_pair: &Pair<Rule>) -> Opcode {
    Opcode::from_str(opcode_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_field(field_pair: Pair<Rule>) -> Field {
    let mut inner_pairs = field_pair.into_inner();
    let mut next_pair = inner_pairs
        .next()
        .expect("Attempt to parse 'field' with no inner pairs");

    let address_mode = if next_pair.as_rule() == Rule::mode {
        let mode = AddressMode::from_str(next_pair.as_str()).unwrap();
        next_pair = inner_pairs
            .next()
            .expect("Attempt to parse 'field' with 'mode' pair but nothing else");
        mode
    } else {
        AddressMode::default()
    };

    Field {
        address_mode,
        value: parse_value(&next_pair),
    }
}

fn parse_value(value_pair: &Pair<Rule>) -> i32 {
    i32::from_str_radix(value_pair.as_str(), 10).unwrap()
}

mod tests {
    // seems to be a case of https://github.com/rust-lang/rust/issues/45268
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn parse_empty() {
        let result = parse("");
        assert!(result.is_err());

        assert_eq!(result.unwrap_err().details, "No input found");
    }

    #[test]
    fn parse_simple_file() {
        let mut expected_core = Core::default();

        expected_core.set(
            0,
            Instruction::new(Opcode::Mov, Field::direct(1), Field::direct(3)),
        );
        expected_core.set(
            1,
            Instruction::new(Opcode::Mov, Field::direct(100), Field::immediate(12)),
        );
        expected_core.set(2, Instruction::default());
        expected_core.set(
            3,
            Instruction::new(Opcode::Jmp, Field::direct(123), Field::direct(45)),
        );

        let simple_input = "
            mov 1, 3
            mov 100, #12
            dat 0, 0
            jmp 123, 45
        ";

        assert_eq!(parse(simple_input).unwrap(), expected_core);
    }
}
