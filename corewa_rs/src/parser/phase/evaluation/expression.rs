//! Helper functions for evaluating an expression syntax tree
//! Most functions here panic instead of returning Result because at this point
//! any errors should have been caught earlier during initial parsing.

use crate::load_file::Offset;
use crate::parser::grammar::*;

/// Evaluate an Expression. Panics if the expression tree is invalid, which
/// should only happen due to programmer error (either the grammar or this code
/// is incorrect).
pub fn evaluate(pair: Pair) -> Offset {
    let mut result = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::Value => result = Some(evaluate_value(inner_pair)),
            Rule::BooleanOp => todo!(),
            other => unreachable!("unexpected rule in Expression: {:?}", other),
        }
    }

    result.unwrap_or_else(|| panic!("Invalid Expression"))
}

fn evaluate_value(pair: Pair) -> Offset {
    let mut result = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::Sum => result = Some(evaluate_sum(inner_pair)),
            Rule::CompareOp => todo!(),
            other => unreachable!("unexpected rule in Value: {:?}", other),
        }
    }

    result.unwrap_or_else(|| panic!("Invalid Value"))
}

fn evaluate_sum(pair: Pair) -> Offset {
    let mut result = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::Product => result = Some(evaluate_product(inner_pair)),
            Rule::AddOp => todo!(),
            other => unreachable!("unexpected rule in SUm: {:?}", other),
        }
    }

    result.unwrap_or_else(|| panic!("Invalid Value"))
}

fn evaluate_product(pair: Pair) -> Offset {
    let mut result = None;

    let mut inner_pairs = pair.into_inner();
    while let Some(inner_pair) = inner_pairs.next() {
        match inner_pair.as_rule() {
            Rule::UnaryExpr => result = Some(evaluate_unary(inner_pair)),
            Rule::MultiplyOp => {
                assert!(
                    result.is_some(),
                    "MultiplyOp encountered before finding first operand"
                );

                let next_pair = inner_pairs.next().unwrap_or_else(|| {
                    panic!("No second operand found in product after {:?}", &inner_pair)
                });
                assert_eq!(next_pair.as_rule(), Rule::UnaryExpr);
                let operand = evaluate_unary(next_pair);

                match inner_pair.as_str() {
                    "*" => result = result.map(|x| x * operand),
                    "/" => result = result.map(|x| x / operand),
                    "%" => result = result.map(|x| x % operand),
                    _ => unreachable!("Invalid MultiplyOp {:?}",),
                }
            }
            other => unreachable!("unexpected rule in SUm: {:?}", other),
        }
    }

    result.unwrap_or_else(|| panic!("Invalid Value"))
}

fn evaluate_unary(pair: Pair) -> Offset {
    let mut result = None;
    let mut unary_ops: Vec<fn(Offset) -> Offset> = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::Number => result = Some(evaluate_number(inner_pair)),
            Rule::Expression => result = Some(evaluate(inner_pair)),
            Rule::UnaryOp => match inner_pair.as_str() {
                "-" => unary_ops.push(|x| -x),
                "+" => (), // Identity function
                "!" => unary_ops.push(|x| if x == 0 { 1 } else { 0 }),
                other => unreachable!("Invalid unary operator {:?}", other),
            },
            other => unreachable!("unexpected {:?} in UnaryExpr", other),
        }
    }

    for op in unary_ops.into_iter() {
        result = result.map(op);
    }

    result.unwrap_or_else(|| panic!("UnaryExpr did not contain a value"))
}

fn evaluate_number(pair: Pair) -> Offset {
    assert!(pair.as_rule() == Rule::Number);
    Offset::from_str_radix(pair.as_str(), 10)
        .unwrap_or_else(|_| panic!("Invalid Number: {:?}", pair.as_str()))
}

#[cfg(test)]
mod test {
    use super::*;

    use test_case::test_case;

    #[test_case("10" => 10; "number")]
    #[test_case("-47" => -47; "negative number")]
    #[test_case("! 10" => 0; "logical not true")]
    #[test_case("! 0" => 1; "logical not false")]
    #[test_case("2 * 3" => 6; "product")]
    #[test_case("24 / 4 / 2" => 3; "multiple quotient")]
    #[test_case("24 % 15 % 8" => 1; "multiple modulo")]
    fn evaluates_expressions(input: &str) -> Offset {
        let pair = parse_expression(input).expect("Failed to parse as Expression");

        evaluate(pair)
    }
}
