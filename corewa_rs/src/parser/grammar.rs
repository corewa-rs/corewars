use pest::{error::Error, iterators::Pairs, Parser as PestParser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar/redcode.pest"]
struct Grammar;

pub fn parse(rule: Rule, input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    Grammar::parse(rule, input)
}

#[cfg(test)]
#[allow(clippy::cognitive_complexity)]
mod tests {
    use pest::{consumes_to, parses_to};

    use super::*;

    // NOTE: these "doctests" are not actually compiled or run
    // since they are in a #[cfg(test)] module

    /// A macro to assert on the way a certain input string parses
    /// Two forms are allowed. One has no identifier:
    /// ```rust
    /// match_parse!(Field {
    ///     "123" | "4567" => [
    ///         // This should look like the `tokens` field of `parses_to!`
    ///     ],
    /// });
    /// ```
    ///
    /// The other allows you to bind the input string so you can use it in your
    /// ```rust
    /// match_parse!(input, Field {
    ///     "123" | "4567" => [
    ///         // You can do something with e.g. `input.len()` here, which
    ///         // will be either 3 or 4 depending on the test case
    ///     ],
    /// });
    /// ```
    macro_rules! match_parse {
        ($rule:ident $args:tt) => {
            match_parse!(_input, $rule $args)
        };
        ($value:ident, $rule:ident {
            $( $($input:tt)|* => $expected:tt ),* $(,)?
        }) => {
            $(
                for $value in [$($input,)*].iter() {
                    parses_to! {
                        parser: Grammar,
                        input: dbg!($value),
                        rule: Rule::$rule,
                        tokens: $expected
                    };
                }
            )*
        };
    }

    #[test]
    fn parse_field() {
        match_parse!(Field {
            "123" => [
                Field(0, 3, [
                    Expr(0, 3, [
                        Number(0, 3),
                    ]),
                ]),
            ],
        });
    }

    #[test]
    fn parse_field_with_mode() {
        match_parse!(Field {
            "#123" | "$123" | "*123" | "@123" | "{123" | "<123" | "}123" | ">123" => [
                Field(0, 4, [
                    AddressMode(0, 1),
                    Expr(1, 4, [
                        Number(1, 4),
                    ]),
                ]),
            ],
        });
    }

    #[test]
    fn parse_expr() {
        // TODO: expand grammar for math operations, parens, etc.
        // Then test it here. Possibly worth breaking into its own module
        match_parse!(Expr {
            "123" => [
                Expr(0, 3, [
                    Number(0, 3),
                ]),
            ]
        });
    }

    #[test]
    fn parse_label_expr() {
        match_parse!(Expr {
            "foo" | "fo2" | "f_2" => [
                Expr(0, 3, [
                    Label(0, 3),
                ]),
            ]
        });
    }

    #[test]
    fn parse_label() {
        match_parse!(label_input, LabelDeclaration {
            "some_label" | "some_label2" => [Label(0, label_input.len())],
            "a: " => [Label(0, 1)],
            " a " => [Label(1, 2)],
            "a :" => [Label(0, 1)],
        });
    }

    #[test]
    fn parse_opcode_modifier() {
        match_parse!(input, Operation {
            "mov.a" | "mov.b" | "mov.ab" | "mov.ba" | "mov.f" | "mov.x" | "mov.i" => [
                Operation(0, input.len(), [
                    Opcode(0, 3),
                    Modifier(4, input.len()),
                ]),
            ],
        });
    }

    #[test]
    fn parse_instruction() {
        match_parse!(Instruction {
            "mov #1, 3" => [
                Instruction(0, 9, [
                    Operation(0, 3, [
                        Opcode(0, 3),
                    ]),
                    Field(4, 6, [
                        AddressMode(4, 5),
                        Expr(5, 6, [
                            Number(5, 6),
                        ]),
                    ]),
                    Field(8, 9, [
                        Expr(8, 9, [
                        Number(8, 9),
                        ])
                    ]),
                ]),
            ],
        });
    }

    #[test]
    fn parse_full_line() {
        match_parse!(input, Line {
            "mov #1, 3; comment" | "mov #1, 3" => [
                Instruction(0, 9, [
                    Operation(0, 3, [
                        Opcode(0, 3),
                    ]),
                    Field(4, 6, [
                        AddressMode(4, 5),
                        Expr(5, 6, [
                            Number(5, 6),
                        ]),
                    ]),
                    Field(8, 9, [
                        Expr(8, 9, [
                            Number(8, 9),
                        ]),
                    ]),
                ]),
            ],
            "lbl mov #1, 3; comment" | "lbl mov #1, 3" => [
                Label(0, 3),
                Instruction(4, 13, [
                    Operation(4, 7, [
                        Opcode(4, 7),
                    ]),
                    Field(8, 10, [
                        AddressMode(8, 9),
                        Expr(9, 10, [
                            Number(9, 10),
                        ]),
                    ]),
                    Field(12, 13, [
                        Expr(12, 13, [
                            Number(12, 13),
                        ]),
                    ]),
                ]),
            ],
        });
    }

    #[test]
    fn parse_comment() {
        match_parse!(input, COMMENT {
            "; foo bar" | ";" => [COMMENT(0, input.len())],
            "; foo bar\n" | ";\n" => [COMMENT(0, input.len() - 1)],
        });
    }

    #[test]
    fn parse_end() {
        match_parse!(input, EndProgram {
            "end" | "END" | "end   " => [
                EndProgram(0, input.len())
            ],
            "end; foo bar" => [
                EndProgram(0, input.len(), [
                    COMMENT(3, input.len()),
                ]),
            ],
            "end\n" | "END\n" | "end   \n" => [
                EndProgram(0, input.len() - 1),
            ],
            "end; foo bar\n" => [
                EndProgram(0, input.len() - 1, [
                    COMMENT(3, input.len() - 1),
                ]),
            ],
        });
    }

    #[test]
    fn parse_end_with_operand() {
        match_parse!(input, EndProgram {
            "end 12345" => [
                EndProgram(0, input.len(), [
                    Expr(4, input.len(), [
                        Number(4, input.len()),
                    ]),
                ]),
            ],
            "end start" => [
                EndProgram(0, input.len(), [
                    Expr(4, input.len(), [
                        Label(4, input.len()),
                    ]),
                ]),
            ],
            "end 12345\n" => [
                EndProgram(0, input.len() - 1, [
                    Expr(4, input.len() - 1, [
                        Number(4, input.len() - 1),
                    ]),
                ]),
            ],
            "end start\n" => [
                EndProgram(0, input.len() - 1, [
                    Expr(4, input.len() - 1, [
                        Label(4, input.len() - 1),
                    ]),
                ]),
            ],
        });
    }
}
