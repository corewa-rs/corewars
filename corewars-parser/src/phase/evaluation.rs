//! In this phase, string lines are parsed into their memory representation.

mod expression;

use std::convert::TryFrom;
use std::str::FromStr;

use corewars_core::load_file;

use super::super::error::Error;
use super::super::grammar;

/// Convert the text input lines into in-memory data structures
pub fn evaluate(lines: Vec<String>) -> Result<load_file::Instructions, Error> {
    let mut instructions = Vec::with_capacity(lines.len());

    for line in lines {
        if let Some(parse_result) = grammar::parse_line(&line)?.next() {
            match &parse_result.as_rule() {
                grammar::Rule::Instruction => {
                    instructions.push(parse_instruction(parse_result.into_inner())?);
                }
                rule => eprintln!("Unexpected rule {:?}", rule),
            }
        }
    }

    Ok(instructions)
}

/// Parse and evaluate a single expression string to find the entry point to
/// a warrior.
pub fn evaluate_expression(expr: &str) -> Result<u32, Error> {
    let expr_pair = grammar::parse_expression(expr)?;

    let origin = expression::evaluate(expr_pair);

    Ok(u32::try_from(origin)?)
}

#[allow(clippy::option_if_let_else)] // TODO
fn parse_instruction(
    mut instruction_pairs: grammar::Pairs,
) -> Result<load_file::Instruction, Error> {
    let mut operation_pairs = instruction_pairs
        .next()
        .expect("Operation must be first pair after Label in Instruction")
        .into_inner();

    let opcode = parse_opcode(
        &operation_pairs
            .next()
            .expect("Opcode must be first pair in Operation"),
    );

    let maybe_modifier = operation_pairs
        .peek()
        .filter(|pair| pair.as_rule() == grammar::Rule::Modifier)
        .map(|pair| parse_modifier(&pair));

    let a_field = parse_field(
        instruction_pairs
            .next()
            .expect("Field must appear after Opcode"),
    );

    let b_field = instruction_pairs
        .next()
        .filter(|pair| pair.as_rule() == grammar::Rule::Field)
        .map(parse_field);

    if let Some(b_field) = b_field {
        let modifier = maybe_modifier.unwrap_or_else(|| {
            load_file::Modifier::default_88_to_94(
                opcode,
                a_field.address_mode,
                b_field.address_mode,
            )
        });

        Ok(load_file::Instruction {
            opcode,
            modifier,
            a_field,
            b_field,
        })
    } else {
        // Special cases for only one argument. There's not much documentation
        // about this except for the introductory guide:
        // http://vyznev.net/corewar/guide.html#deep_instr
        // Basically just ported from the pMARS reference implementation: src/asm.c:1300
        use load_file::Opcode::{Dat, Jmp, Nop, Spl};

        match opcode {
            Dat => Ok(load_file::Instruction {
                opcode,
                modifier: maybe_modifier.unwrap_or(load_file::Modifier::F),
                a_field: load_file::Field::immediate(0),
                b_field: a_field,
            }),
            Jmp | Spl | Nop => Ok(load_file::Instruction {
                opcode,
                modifier: maybe_modifier.unwrap_or(load_file::Modifier::B),
                a_field,
                b_field: load_file::Field::direct(0),
            }),
            other => Err(Error::InvalidArguments { opcode: other }),
        }
    }
}

fn parse_modifier(modifier_pair: &grammar::Pair) -> load_file::Modifier {
    load_file::Modifier::from_str(modifier_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_opcode(opcode_pair: &grammar::Pair) -> load_file::Opcode {
    load_file::Opcode::from_str(opcode_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_field(field_pair: grammar::Pair) -> load_file::Field {
    let mut field_pairs = field_pair.into_inner();

    let address_mode = field_pairs
        .peek()
        .filter(|pair| pair.as_rule() == grammar::Rule::AddressMode)
        .map_or(load_file::AddressMode::default(), |pair| {
            load_file::AddressMode::from_str(pair.as_str()).expect("Invalid AddressMode")
        });

    let offset = expression::evaluate(
        field_pairs
            .find(|pair| pair.as_rule() == grammar::Rule::Expression)
            .unwrap_or_else(|| panic!("No expression found in Field: {:?}", field_pairs)),
    );

    load_file::Field {
        address_mode,
        value: load_file::Value::Literal(offset),
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use load_file::{Field, Instruction, Opcode};

    #[test]
    fn parse_simple_file() {
        let simple_input = [
            "mov 1, 3",
            "mov 100, #12",
            "dat #0, #0",
            "jmp +123, #45",
            "jmp -4",
            "jmp -1",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();

        let expected_core = vec![
            Instruction::new(Opcode::Mov, Field::direct(1), Field::direct(3)),
            Instruction::new(Opcode::Mov, Field::direct(100), Field::immediate(12)),
            Instruction::new(Opcode::Dat, Field::immediate(0), Field::immediate(0)),
            Instruction::new(Opcode::Jmp, Field::direct(123), Field::immediate(45)),
            Instruction::new(Opcode::Jmp, Field::direct(-4), Field::direct(0)),
            Instruction::new(Opcode::Jmp, Field::direct(-1), Field::direct(0)),
        ];

        let parsed = evaluate(simple_input)
            .unwrap_or_else(|err| panic!("Failed to parse simple file: {}", err));

        assert_eq!(parsed, expected_core);
    }

    #[test]
    fn evaluates_origin() {
        let evaluated = evaluate_expression("2 * (4 + 3)").expect("Should parse successfully");
        assert_eq!(evaluated, 14);
    }

    #[test]
    fn fails_for_negative_origin() {
        evaluate_expression("-10").expect_err("-10 should be an invalid origin");
    }
}
