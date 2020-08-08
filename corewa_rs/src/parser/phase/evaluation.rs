//! In this phase, string lines are parsed into their memory representation.

mod expression;

use std::convert::TryFrom;
use std::str::FromStr;

use crate::error::Error;
use crate::load_file;
use crate::parser::grammar;

/// Convert the text input lines into in-memory data structures
pub fn evaluate(lines: Vec<String>) -> Result<load_file::Instructions, Error> {
    let mut instructions = Vec::with_capacity(lines.len());

    for line in lines.into_iter() {
        if let Some(parse_result) = grammar::parse_line(&line)?.next() {
            match &parse_result.as_rule() {
                grammar::Rule::Instruction => {
                    instructions.push(parse_instruction(parse_result.into_inner()));
                }
                rule => dbgf!("Unexpected rule {:?}", rule),
            }
        }
    }

    Ok(instructions)
}

/// Parse and evaluate a single expression string to find the entry point to
/// a warrior.
pub fn evaluate_origin(expr: String) -> Result<load_file::UOffset, Error> {
    let expr_pair = grammar::parse_expression(&expr)?;

    let origin = expression::evaluate(expr_pair);

    load_file::UOffset::try_from(origin)
        .map_err(|_| Error::new(format!("Invalid origin {}", origin)))
}

fn parse_instruction(mut instruction_pairs: grammar::Pairs) -> load_file::Instruction {
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

    let field_a = parse_field(
        instruction_pairs
            .next()
            .expect("Field must appear after Opcode"),
    );

    let field_b = instruction_pairs
        .next()
        .filter(|pair| pair.as_rule() == grammar::Rule::Field)
        .map(parse_field);

    if let Some(field_b) = field_b {
        let modifier = maybe_modifier.unwrap_or_else(|| {
            load_file::Modifier::default_88_to_94(
                opcode,
                field_a.address_mode,
                field_b.address_mode,
            )
        });

        load_file::Instruction {
            opcode,
            modifier,
            field_a,
            field_b,
        }
    } else {
        // Special cases for only one argument. There's not much documentation
        // about this except for the introductory guide:
        // http://vyznev.net/corewar/guide.html#deep_instr
        // Basically just ported from the pMARS reference implementation: src/asm.c:1300
        use load_file::Opcode::*;

        match opcode {
            Dat => load_file::Instruction {
                opcode,
                modifier: maybe_modifier.unwrap_or(load_file::Modifier::F),
                field_a: load_file::Field::immediate(0),
                field_b: field_a,
            },
            Jmp | Spl | Nop => load_file::Instruction {
                opcode,
                modifier: maybe_modifier.unwrap_or(load_file::Modifier::B),
                field_a,
                field_b: load_file::Field::direct(0),
            },
            other => {
                // TODO #25 real error
                panic!("Expected two arguments for {} but only found one", other)
            }
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
        .map(|s| s.to_string())
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
        let evaluated = evaluate_origin("2 * (4 + 3)".into()).expect("Should parse successfully");
        assert_eq!(evaluated, 14);
    }

    #[test]
    fn fails_for_negative_origin() {
        evaluate_origin("-10".into()).expect_err("-10 should be an invalid origin");
    }
}
