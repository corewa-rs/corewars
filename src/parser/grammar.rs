use pest::{error::Error, iterators::Pairs, Parser};

#[derive(Parser)]
#[grammar = "data/redcode.pest"]
struct Grammar;

pub fn parse(rule: Rule, input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    Grammar::parse(rule, input)
}

#[cfg(test)]
#[allow(clippy::cognitive_complexity)]
mod tests {
    use pest::{consumes_to, parses_to};

    use super::*;

    #[test]
    fn parse_field() {
        parses_to! {
            parser: Grammar,
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
                parser: Grammar,
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
            parser: Grammar,
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
                parser: Grammar,
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
                parser: Grammar,
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
            parser: Grammar,
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
            parser: Grammar,
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
                parser: Grammar,
                input: label_input,
                rule: Rule::LabelDeclaration,
                tokens: [Label(start, end)]
            }
        }
    }
}
