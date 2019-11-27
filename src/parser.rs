use std::{error, fmt, str::FromStr};

use itertools::Itertools;
use pest::{
    error::{Error as PestError, ErrorVariant, LineColLocation},
    iterators::{Pair, Pairs},
    Parser,
};

use crate::load_file::{AddressMode, Core, Field, Instruction, Modifier, Opcode, Value};

#[derive(Debug)]
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

impl From<PestError<Rule>> for Error {
    fn from(pest_error: PestError<Rule>) -> Error {
        Error {
            details: format!(
                "Parse error: {} {}",
                match pest_error.variant {
                    ErrorVariant::ParsingError {
                        positives,
                        negatives,
                    } => format!("expected one of {:?}, none of {:?}", positives, negatives),
                    ErrorVariant::CustomError { message } => message,
                },
                match pest_error.line_col {
                    LineColLocation::Pos((line, col)) => format!("at line {} column {}", line, col),
                    LineColLocation::Span((start_line, start_col), (end_line, end_col)) => format!(
                        "from line {} column {} to line {} column {}",
                        start_line, start_col, end_line, end_col
                    ),
                }
            ),
        }
    }
}

impl From<String> for Error {
    fn from(details: String) -> Error {
        Error { details }
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

    let parse_result = RedcodeParser::parse(Rule::RedcodeFile, file_contents)?
        .next()
        .ok_or_else(Error::no_input)?;

    let mut i = 0;
    for mut line_pair in parse_result
        .into_inner()
        .map(Pair::into_inner)
        .filter(|line_pair| line_pair.peek().is_some())
    {
        let label_pairs = line_pair
            .take_while_ref(|pair| pair.as_rule() == Rule::Label)
            .map(|pair| pair.as_str().to_owned());

        for label in label_pairs {
            core.add_label(i, label.to_string())?;
        }

        if line_pair.peek().is_some() {
            core.set(i, parse_instruction(line_pair));
            i += 1;
        }
    }

    // TODO: keep the original core or use the resolved one?
    // Probably it should keep a resolved copy in itself
    Ok(core.resolve()?)
}

fn parse_instruction(mut instruction_pairs: Pairs<Rule>) -> Instruction {
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
        .filter(|pair| pair.as_rule() == Rule::Modifier)
        .map(|pair| parse_modifier(&pair));

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
        Modifier::default_88_to_94(opcode, field_a.address_mode, field_b.address_mode)
    });

    Instruction {
        opcode,
        modifier,
        field_a,
        field_b,
    }
}

fn parse_modifier(modifier_pair: &Pair<Rule>) -> Modifier {
    Modifier::from_str(modifier_pair.as_str().to_uppercase().as_ref()).unwrap()
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

fn parse_value(value_pair: Pair<Rule>) -> Value {
    let expr_inner = value_pair
        .into_inner()
        .next()
        .expect("Expr must have inner value");

    match expr_inner.as_rule() {
        Rule::Number => Value::Literal(
            i32::from_str_radix(expr_inner.as_str(), 10)
                .expect("Number type must be decimal integer"),
        ),
        Rule::Label => Value::Label(expr_inner.as_str().to_owned()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
#[allow(clippy::cognitive_complexity)]
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
    fn parse_expr() {
        // TODO: expand grammar for math operations, parens, etc.
        // Then test it here. Possibly worth breaking into its own module
        parses_to! {
            parser: RedcodeParser,
            input: "123",
            rule: Rule::Expr,
            tokens: [
                Expr(0, 3, [
                    Number(0, 3)
                ]),
            ]
        };
    }

    #[test]
    fn parse_label_expr() {
        for test_input in ["foo", "fo2", "f_2"].iter() {
            parses_to! {
                parser: RedcodeParser,
                input: test_input,
                rule: Rule::Expr,
                tokens: [
                    Expr(0, 3, [
                        Label(0, 3)
                    ]),
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
    fn parse_label() {
        for &(label_input, start, end) in [
            ("some_label", 0, 10),
            ("some_label2", 0, 11),
            ("a: ", 0, 1),
            (" a ", 1, 2),
            ("a :", 0, 1),
        ]
        .iter()
        {
            parses_to! {
                parser: RedcodeParser,
                input: label_input,
                rule: Rule::LabelDeclaration,
                tokens: [Label(start, end)]
            }
        }
    }

    #[test]
    fn duplicate_labels() {
        let simple_input = "
            label1  dat 0,0
            label1  dat 0,0
        ";

        parse(simple_input).expect_err("Should fail for duplicate label");
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

        let parsed = parse(simple_input).expect("Should parse simple file");

        assert_eq!(parsed, expected_core);
    }

    // TODO: parse error for unresolvable label
}
