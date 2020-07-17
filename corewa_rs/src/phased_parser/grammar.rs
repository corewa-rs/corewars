//! Definition and tests for the grammar that defines a valid line of Redcode.
//! Provides helper function to tokenize strings into span-like tokens.

pub(super) use pest::{
    error::Error,
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "phased_parser/grammar/redcode_line.pest"]
pub(super) struct Grammar;

/// Parse an input line and return an iterator over

pub(super) fn tokenize(line: &str) -> Vec<Pair<Rule>> {
    parse(line).map(flatten_pairs).unwrap_or_default()
}

pub(super) fn parse(line: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    Grammar::parse(Rule::Line, line)
}

fn flatten_pairs(pairs: Pairs<Rule>) -> Vec<Pair<Rule>> {
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
    fn parse_full_line() {
        match_parse!(input, Line {
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
