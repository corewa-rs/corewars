//! Helper functions for evaluating an expression syntax tree.
//!
//! Most functions here panic instead of returning Result because at this point
//! any errors should have been caught earlier during initial parsing.

use corewars_core::load_file::Offset;

use crate::grammar::*;

/// Evaluate an Expression. Panics if the expression tree is invalid, which
/// should only happen due to programmer error (either the grammar or this code
/// is incorrect).
pub fn evaluate(pair: Pair) -> Offset {
    let mut result = None;
    let mut boolean_op: fn(Offset, Offset) -> Offset =
        |_, _| unreachable!("BooleanOp called before first operand");

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::Value => {
                let operand = evaluate_value(inner_pair);
                result = result.map(|x| boolean_op(x, operand)).or(Some(operand));
            }
            Rule::BooleanOp => {
                boolean_op = match inner_pair.as_str() {
                    "&&" => |a, b| (a != 0 && b != 0) as Offset,
                    "||" => |a, b| (a != 0 || b != 0) as Offset,
                    op => unreachable!("Invalid BooleanOp {:?}", op),
                }
            }
            other => unreachable!("unexpected rule in Expression: {:?}", other),
        }
    }

    result.unwrap_or_else(|| panic!("Invalid Expression"))
}

fn evaluate_value(pair: Pair) -> Offset {
    let mut result = None;
    let mut compare_op: fn(Offset, Offset) -> Offset =
        |_, _| unreachable!("CompareOp called before first operand");

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::Sum => {
                let operand = evaluate_sum(inner_pair);
                result = result.map(|x| compare_op(x, operand)).or(Some(operand));
            }
            Rule::CompareOp => {
                // Casting bool to integer is always 0 or 1
                compare_op = match inner_pair.as_str() {
                    ">" => |a, b| (a > b) as Offset,
                    ">=" => |a, b| (a >= b) as Offset,
                    "<" => |a, b| (a < b) as Offset,
                    "<=" => |a, b| (a <= b) as Offset,
                    "==" => |a, b| (a == b) as Offset,
                    "!=" => |a, b| (a != b) as Offset,
                    op => unreachable!("Invalid CompareOp {:?}", op),
                };
            }
            other => unreachable!("unexpected rule in Value: {:?}", other),
        }
    }

    result.unwrap_or_else(|| panic!("Invalid Value"))
}

fn evaluate_sum(pair: Pair) -> Offset {
    let mut result = None;
    let mut add_op: fn(Offset, Offset) -> Offset =
        |_, _| unreachable!("AddOp called before first operand");

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::Product => {
                let operand = evaluate_product(inner_pair);
                result = result.map(|x| add_op(x, operand)).or(Some(operand));
            }
            Rule::AddOp => {
                add_op = match inner_pair.as_str() {
                    "+" => |a, b| a + b,
                    "-" => |a, b| a - b,
                    op => unreachable!("Invalid AddOp {:?}", op),
                };
            }
            rule => unreachable!("unexpected {:?} in Sum: {:?}", rule, inner_pair.as_str()),
        }
    }

    result.unwrap_or_else(|| panic!("Invalid Sum"))
}

fn evaluate_product(pair: Pair) -> Offset {
    let mut result = None;
    let mut mul_op: fn(Offset, Offset) -> Offset =
        |_, _| unreachable!("MultiplyOp called before first operand");

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::UnaryExpr => {
                let operand = evaluate_unary(inner_pair);
                result = result.map(|x| mul_op(x, operand)).or(Some(operand));
            }
            Rule::MultiplyOp => {
                mul_op = match inner_pair.as_str() {
                    "*" => |a, b| a * b,
                    "/" => |a, b| a / b,
                    "%" => |a, b| a % b,
                    op => unreachable!("Invalid MultiplyOp {:?}", op),
                };
            }
            rule => unreachable!(
                "unexpected {:?} in Product: {:?}",
                rule,
                inner_pair.as_str()
            ),
        }
    }

    result.unwrap_or_else(|| panic!("Invalid Product"))
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
                "!" => unary_ops.push(|x| (x == 0) as Offset),
                other => unreachable!("Invalid unary operator {:?}", other),
            },
            other => unreachable!(
                "unexpected {:?} in UnaryExpr: {:?}",
                other,
                inner_pair.as_str()
            ),
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

    // Unary/atoms
    #[test_case("10" => 10; "number")]
    #[test_case("-47" => -47; "negative number")]
    #[test_case("! 10" => 0; "boolean not true")]
    #[test_case("! 0" => 1; "boolean not false")]
    // Product
    #[test_case("2 * 3" => 6; "product")]
    #[test_case("24 / 4 / 2" => 3; "multiple quotient")]
    #[test_case("24 % 15 % 6" => 3; "multiple modulo")]
    // Sum
    #[test_case("1 + 1" => 2; "sum")]
    #[test_case("1 + 2 * 3" => 7; "sum and product")]
    // Parentheses
    #[test_case("(2)" => 2; "parenthesized number")]
    #[test_case("(2 * 3)" => 6; "parenthesized product")]
    #[test_case("-(2 * 3)" => -6; "unary parenthesized product")]
    #[test_case("(2 + 3)" => 5; "parenthesized sum")]
    // Comparison
    #[test_case("1 < 2" => 1; "less than")]
    #[test_case("1 <= 2" => 1; "less than or equal")]
    #[test_case("1 > 2" => 0; "greater than")]
    #[test_case("1 >= 2" => 0; "greater than or equal")]
    #[test_case("1 == 2" => 0; "equals")]
    #[test_case("1 != 2" => 1; "not equals")]
    // Boolean
    #[test_case("0 && 1" => 0; "boolean and")]
    #[test_case("0 || 1" => 1; "boolean or")]
    fn evaluates_expressions(input: &str) -> Offset {
        let pair = parse_expression(input).expect("Failed to parse as Expression");

        evaluate(pair)
    }
}
