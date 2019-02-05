use std::{convert::From, fmt, str::FromStr};

use pest::{
    error::Error as PestError,
    iterators::{Pair, Pairs},
    Parser,
};

use crate::load_file::{AddressMode, Core, Field, Instruction, Modifier, Opcode};

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

    let parse_result = RedcodeParser::parse(Rule::AssemblyFile, file_contents)?
        .next()
        .ok_or_else(Error::no_input)?;

    for (i, line_pairs) in parse_result
        .into_inner()
        .map(Pair::into_inner)
        .filter(|line_pair| line_pair.peek().is_some())
        .enumerate()
    {
        core.set(i, parse_instruction(line_pairs));
    }

    Ok(core)
}

fn parse_instruction(mut instruction_pairs: Pairs<Rule>) -> Instruction {
    let opcode = parse_opcode(
        &instruction_pairs
            .next()
            .expect("Opcode must be first pair in Instruction"),
    );

    let maybe_modifier = instruction_pairs
        .peek()
        .filter(|pair| pair.as_rule() == Rule::Modifier)
        .map(|pair| Modifier::from_str(pair.as_str()).expect("Invalid Modifier"));

    let field_a = parse_field(
        instruction_pairs
            .next()
            .expect("Field must appear after Opcode"),
    );

    let field_b = instruction_pairs
        .next()
        .filter(|pair| pair.as_rule() == Rule::Field)
        .map_or_else(Field::default, parse_field);

    let modifier = maybe_modifier.unwrap_or_else(|| {
        Modifier::from_context(opcode, field_a.address_mode, field_b.address_mode)
    });

    Instruction {
        opcode,
        modifier,
        field_a,
        field_b,
    }
}

fn parse_opcode(opcode_pair: &Pair<Rule>) -> Opcode {
    Opcode::from_str(opcode_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_field(field_pair: Pair<Rule>) -> Field {
    let field_pairs = field_pair.into_inner();

    let address_mode = field_pairs
        .peek()
        .filter(|pair| pair.as_rule() == Rule::AddressMode)
        .map_or(AddressMode::default(), |pair| {
            AddressMode::from_str(pair.as_str()).expect("Invalid AddressMode")
        });

    let value = parse_value(
        field_pairs
            .skip_while(|pair| pair.as_rule() != Rule::Expr)
            .next()
            .expect("No Expr in Field"),
    );

    Field {
        address_mode,
        value,
    }
}

fn parse_value(value_pair: Pair<Rule>) -> i32 {
    i32::from_str_radix(value_pair.as_str(), 10).unwrap()
}

#[cfg(test)]
mod tests {
    use pest::{consumes_to, parses_to};

    use super::*;

    #[test]
    fn parse_empty() {
        let result = parse("");
        assert!(result.is_err());

        assert_eq!(result.unwrap_err().details, "No input found");
    }

    #[test]
    fn parse_field() {
        parses_to! {
            parser: RedcodeParser,
            input: "123",
            rule: Rule::Field,
            tokens: [
                Field(0, 3, [
                    Expr(0, 3, [
                        Number(0, 3)
                    ]),
                ])
            ]
        };
    }

    #[test]
    fn parse_field_with_mode() {
        for test_input in [
            "#123", "$123", "*123", "@123", "{123", "<123", "}123", ">123",
        ]
        .iter()
        {
            parses_to! {
                parser: RedcodeParser,
                input: test_input,
                rule: Rule::Field,
                tokens: [
                    Field(0, 4, [
                        AddressMode(0, 1),
                        Expr(1, 4, [
                            Number(1, 4)
                        ]),
                    ])
                ]
            };
        }
    }

    #[test]
    fn parse_opcode_modifier() {
        for test_input in [
            "mov.a", "mov.b", "mov.ab", "mov.ba", "mov.f", "mov.x", "mov.i",
        ]
        .iter()
        {
            parses_to! {
                parser: RedcodeParser,
                input: test_input,
                rule: Rule::Operation,
                tokens: [
                    Operation(0, test_input.len(), [
                        Opcode(0, 3),
                        Modifier(4, test_input.len()),
                    ]),
                ]
            }
        }
    }

    #[allow(clippy::cyclomatic_complexity)]
    #[test]
    fn parse_instruction() {
        parses_to! {
            parser: RedcodeParser,
            input: "mov #1, 3",
            rule: Rule::Instruction,
            tokens: [
                Operation(0, 3, [
                    Opcode(0, 3)
                ]),
                Field(4, 6, [
                    AddressMode(4, 5),
                    Expr(5, 6, [
                        Number(5, 6)
                    ])
                ]),
                Field(8, 9, [
                    Expr(8, 9, [
                       Number(8, 9)
                    ])
                ]),
            ]
        };
    }

    #[test]
    fn parse_comment() {
        parses_to! {
            parser: RedcodeParser,
            input: "; foo bar\n",
            rule: Rule::COMMENT,
            tokens: [
                COMMENT(0, 9)
            ]
        }
    }

    #[test]
    fn parse_simple_file() {
        let simple_input = "
            mov 1, 3 ; make sure comments parse out
            mov 100, #12
            dat 0, 0
            jmp 123, 45
        ";

        let mut expected_core = Core::default();

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
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0)),
        );
        expected_core.set(
            3,
            Instruction::new(Opcode::Jmp, Field::direct(123), Field::direct(45)),
        );

        assert_eq!(parse(simple_input).unwrap(), expected_core);
    }
}
