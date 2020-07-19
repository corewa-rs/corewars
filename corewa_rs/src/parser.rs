use std::{error, fmt, str::FromStr};

use pest::iterators::{Pair, Pairs};

use crate::load_file::{AddressMode, Field, Instruction, Modifier, Opcode, Program, Value};

mod grammar;

#[derive(Debug, PartialEq)]
pub struct Error {
    details: String,
}

impl error::Error for Error {}

impl fmt::Display for Error {
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

pub trait IntoError: fmt::Display {}

impl<T: pest::RuleType> IntoError for pest::error::Error<T> {}
impl IntoError for String {}
impl IntoError for &str {}

impl<T: IntoError> From<T> for Error {
    fn from(displayable_error: T) -> Self {
        Error {
            details: displayable_error.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ParsedProgram {
    pub result: Program,
    pub warnings: Vec<Error>,
}

impl fmt::Display for ParsedProgram {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.result)
    }
}

pub fn parse(file_contents: &str) -> Result<ParsedProgram, Error> {
    if file_contents.is_empty() {
        return Err(Error::no_input());
    }

    let mut warnings = Vec::new();

    let mut program = Program::new();

    let parse_result = grammar::parse(grammar::Rule::Program, file_contents)?
        .next()
        .ok_or_else(Error::no_input)?;

    let mut i = 0;
    for pair in parse_result
        .into_inner()
        .take_while(|pair| pair.as_rule() != grammar::Rule::EndProgram)
    {
        match &pair.as_rule() {
            grammar::Rule::Label => {
                if let Err(failed_add) = program.add_label(i, pair.as_str().to_string()) {
                    warnings.push(failed_add.into());
                }
            }
            grammar::Rule::Instruction => {
                let instruction = parse_instruction(pair.into_inner());

                if instruction.opcode == Opcode::Org {
                    program.set_origin(instruction.field_a);
                } else {
                    program.set(i, instruction);
                    i += 1;
                }
            }
            _ => (),
        }
    }

    Ok(ParsedProgram {
        result: program,
        warnings,
    })
}

fn parse_instruction(mut instruction_pairs: Pairs<grammar::Rule>) -> Instruction {
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
        .map_or_else(Field::default, parse_field);

    let modifier = maybe_modifier.unwrap_or_else(|| {
        Modifier::default_88_to_94(opcode, field_a.address_mode, field_b.address_mode)
    });

    Instruction {
        opcode,
        modifier,
        field_a,
        field_b,
    }
}

fn parse_modifier(modifier_pair: &Pair<grammar::Rule>) -> Modifier {
    Modifier::from_str(modifier_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_opcode(opcode_pair: &Pair<grammar::Rule>) -> Opcode {
    Opcode::from_str(opcode_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_field(field_pair: Pair<grammar::Rule>) -> Field {
    let mut field_pairs = field_pair.into_inner();

    let address_mode = field_pairs
        .peek()
        .filter(|pair| pair.as_rule() == grammar::Rule::AddressMode)
        .map_or(AddressMode::default(), |pair| {
            AddressMode::from_str(pair.as_str()).expect("Invalid AddressMode")
        });

    let value = parse_value(
        field_pairs
            .find(|pair| pair.as_rule() == grammar::Rule::Expr)
            .expect("No Expr in Field"),
    );

    Field {
        address_mode,
        value,
    }
}

fn parse_value(value_pair: Pair<grammar::Rule>) -> Value {
    let expr_inner = value_pair
        .into_inner()
        .next()
        .expect("Expr must have inner value");

    match expr_inner.as_rule() {
        grammar::Rule::Number => Value::Literal(
            i32::from_str_radix(expr_inner.as_str(), 10)
                .expect("Number type must be decimal integer"),
        ),
        grammar::Rule::Label => Value::Label(expr_inner.as_str().to_owned()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let parsed = parse("");
        assert!(parsed.is_err());

        assert_eq!(parsed.unwrap_err().details, "No input found");
    }

    #[test]
    fn duplicate_labels() {
        let simple_input = "
            label1  dat 0,0
            label1  dat 0,0
        ";

        let parsed = parse(simple_input).expect("Failed to parse");

        assert_eq!(
            parsed.warnings,
            vec![Error::from("Label 'label1' already exists")]
        );

        assert_eq!(parsed.result.label_address("label1"), Some(1));
    }

    #[test]
    fn parse_simple_file() {
        let simple_input = "
            preload
            begin:  mov 1, 3 ; make sure comments parse out
                    mov 100, #12
            loop:
            main    dat #0, #0
                    jmp +123, #45
                    jmp begin
                    jmp -1
        ";

        let mut expected_core = Program::default();

        expected_core.set(
            0,
            Instruction::new(Opcode::Mov, Field::direct(1), Field::direct(3)),
        );
        expected_core.set(
            1,
            Instruction::new(Opcode::Mov, Field::direct(100), Field::immediate(12)),
        );
        expected_core.set(
            2,
            Instruction::new(Opcode::Dat, Field::immediate(0), Field::immediate(0)),
        );
        expected_core.set(
            3,
            Instruction::new(Opcode::Jmp, Field::direct(123), Field::immediate(45)),
        );
        expected_core.set(
            4,
            Instruction::new(Opcode::Jmp, Field::direct(-4), Field::immediate(0)),
        );
        expected_core.set(
            5,
            Instruction::new(Opcode::Jmp, Field::direct(-1), Field::immediate(0)),
        );

        expected_core
            .resolve()
            .expect("Should resolve a core with no labels");

        let mut parsed = parse(simple_input)
            .unwrap_or_else(|err| panic!("Failed to parse simple file: {}", err));

        parsed.result.resolve().expect("Parsed file should resolve");

        assert!(parsed.warnings.is_empty());

        assert_eq!(parsed.result, expected_core);
    }
}
