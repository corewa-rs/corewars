//! Definition and tests for the grammar that defines a valid line of Redcode.
//! Provides helper function to tokenize strings into span-like tokens.

use pest::{
    error::Error as PestError,
    iterators::{Pair as PestPair, Pairs as PestPairs},
    Parser,
};
use pest_derive::Parser;

use crate::error::Error;

pub type Pair<'a> = PestPair<'a, Rule>;
pub type Pairs<'a> = PestPairs<'a, Rule>;

#[derive(Parser)]
#[grammar = "parser/grammar/redcode.pest"]
pub struct Grammar;

/// Parse an input line and return an iterator over

pub fn tokenize(line: &str) -> Vec<Pair> {
    parse_line(line).map(flatten_pairs).unwrap_or_default()
}

pub fn parse_line(line: &str) -> Result<Pairs, PestError<Rule>> {
    Grammar::parse(Rule::Line, line)
}

pub fn parse_expression(line: &str) -> Result<Pair, Error> {
    let mut pairs = Grammar::parse(Rule::Expression, line)?;
    pairs
        .find(|pair| pair.as_rule() == Rule::Expression)
        .ok_or_else(|| Error::new(format!("No expression found in line: {:?}", line)))
}

fn flatten_pairs(pairs: Pairs) -> Vec<Pair> {
    pairs
        .flatten()
        .filter(|pair|
            // TODO avoid clone here if possible
            pair.clone().into_inner().peek().is_none())
        .collect()
}

#[cfg(any(test, doctest))] // cfg(doctest) so we run the helper's doctest
mod test {
    use pest::{consumes_to, parses_to};
    use test_case::test_case;

    use super::*;
    use Rule::*;

    /// A macro to assert on the way a certain input string parses
    /// Two forms are allowed. One has no identifier:
    /// ```
    /// match_parse!(Field {
    ///     "123" | "4567" => [
    ///         // This should look like the `tokens` field of `parses_to!`
    ///     ],
    /// });
    /// ```
    ///
    /// The other allows you to bind the input string so you can use it in your
    /// ```
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
                        input: $value,
                        rule: Rule::$rule,
                        tokens: $expected
                    };
                }
            )*
        };
    }

    #[test]
    fn parse_expression() {
        match_parse!(Expression {
            "123" => [
                Expression(0, 3, [
                    Value(0, 3, [
                        Sum(0, 3, [
                            Product(0, 3, [
                                UnaryExpr(0, 3, [
                                    Number(0, 3)
                                ])
                            ])
                        ]),
                    ]),
                ]),
            ],
            "-10" => [
                Expression(0, 3, [
                    Value(0, 3, [
                        Sum(0, 3, [
                            Product(0, 3, [
                                UnaryExpr(0, 3, [
                                    UnaryOp(0, 1),
                                    Number(1, 3)
                                ])
                            ])
                        ]),
                    ]),
                ]),
            ],
            "2 + 2" => [
                Expression(0, 5, [
                    Value(0, 5, [
                        Sum(0, 5, [
                            Product(0, 2, [
                                UnaryExpr(0, 1, [
                                    Number(0, 1)
                                ]),
                            ]),
                            AddOp(2, 3),
                            Product(4, 5, [
                                UnaryExpr(4, 5, [
                                    Number(4, 5)
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ],
            "2 + -2" => [
                Expression(0, 6, [
                    Value(0, 6, [
                        Sum(0, 6, [
                            Product(0, 2, [
                                UnaryExpr(0, 1, [
                                    Number(0, 1)
                                ]),
                            ]),
                            AddOp(2, 3),
                            Product(4, 6, [
                                UnaryExpr(4, 6, [
                                    UnaryOp(4, 5),
                                    Number(5, 6),
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ],
            "2*(x + 1)" => [
                Expression(0, 9, [
                    Value(0, 9, [
                        Sum(0, 9, [
                            Product(0, 9, [
                                UnaryExpr(0, 1, [
                                    Number(0, 1)
                                ]),
                                MultiplyOp(1, 2),
                                UnaryExpr(2, 9, [
                                    Expression(3, 8, [
                                        Value(3, 8, [
                                            Sum(3, 8, [
                                                Product(3, 5, [
                                                    UnaryExpr(3, 4, [
                                                        Label(3, 4),
                                                    ]),
                                                ]),
                                                AddOp(5, 6),
                                                Product(7, 8, [
                                                    UnaryExpr(7, 8, [
                                                        Number(7, 8)
                                                    ]),
                                                ]),
                                            ]),
                                        ]),
                                    ]),
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ],
            "x >= 2 || x < 0" => [
                Expression(0, 15, [
                    Value(0, 7, [
                        Sum(0, 2, [
                            Product(0, 2, [
                                UnaryExpr(0, 1, [
                                    Label(0, 1),
                                ]),
                            ]),
                        ]),
                        CompareOp(2, 4),
                        Sum(5, 7, [
                            Product(5, 7, [
                                UnaryExpr(5, 6, [
                                    Number(5, 6),
                                ]),
                            ]),
                        ]),
                    ]),
                    BooleanOp(7, 9),
                    Value(10, 15, [
                        Sum(10, 12, [
                            Product(10, 12, [
                                UnaryExpr(10, 11, [
                                    Label(10, 11),
                                ]),
                            ]),
                        ]),
                        CompareOp(12, 13),
                        Sum(14, 15, [
                            Product(14, 15, [
                                UnaryExpr(14, 15, [
                                    Number(14, 15),
                                ]),
                            ]),
                        ]),
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
                    Expression(1, 4, [
                        Value(1, 4, [
                            Sum(1, 4, [
                                Product(1, 4, [
                                    UnaryExpr(1, 4, [
                                        Number(1, 4)
                                    ])
                                ])
                            ]),
                        ]),
                    ]),
                ])
            ],
        });
    }

    #[test]
    fn parse_label_expr() {
        match_parse!(Expression {
            "foo" | "fo2" | "f_2" => [
                Expression(0, 3, [
                    Value(0, 3, [
                        Sum(0, 3, [
                            Product(0, 3, [
                                UnaryExpr(0, 3, [
                                    Label(0, 3)
                                ])
                            ])
                        ]),
                    ]),
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
    fn parse_full_line() {
        match_parse!(input, Line {
            "mov #1, 3" => [
                Instruction(0, 9, [
                    Operation(0, 3, [
                        Opcode(0, 3),
                    ]),
                    Field(4, 6, [
                        AddressMode(4, 5),
                        Expression(5, 6, [
                            Value(5, 6, [
                                Sum(5, 6, [
                                    Product(5, 6, [
                                        UnaryExpr(5, 6, [
                                            Number(5, 6)
                                        ])
                                    ])
                                ]),
                            ]),
                        ]),
                    ]),
                    Field(8, 9, [
                        Expression(8, 9, [
                            Value(8, 9, [
                                Sum(8, 9, [
                                    Product(8, 9, [
                                        UnaryExpr(8, 9, [
                                            Number(8, 9)
                                        ])
                                    ])
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ],
        });
    }

    #[test_case("lbl", vec![(Label, "lbl")]; "label")]
    #[test_case("lbl: ", vec![(Label, "lbl")]; "label with colon")]
    #[test_case(
        "lbl: mov 0, 1",
        vec![
            (Label, "lbl"),
            (Opcode, "mov"),
            (Number, "0"),
            (Number, "1"),
        ];
        "label instruction"
    )]
    #[test_case(
        "lbl equ 4",
        vec![(Label, "lbl"), (Substitution, "4")];
        "label equ expr"
    )]
    #[test_case(
        "lbl equ mov 1, 2",
        vec![(Label, "lbl"), (Substitution, "mov 1, 2")];
        "label equ instruction"
    )]
    #[test_case(
        "equ mov 1, 2",
        vec![(Substitution, "mov 1, 2")];
        "equ continuation"
    )]
    #[test_case(
        "equ mov 1, (1 + 2)",
        vec![(Substitution, "mov 1, (1 + 2)")];
        "equ continuation expr"
    )]
    fn tokenize_line(input: &str, expected_result: Vec<(Rule, &str)>) {
        let actual: Vec<(Rule, &str)> = tokenize(input)
            .iter()
            .map(|pair| (pair.as_rule(), pair.as_str()))
            .collect();

        assert_eq!(actual, expected_result);
    }
}
