//! This phase finds and expands substitutions, namely:
//! - EQU definitions
//! - FOR blocks
//! - Standard labels which alias an address
//!
//! Labels used in the right-hand side of an expression are not substituted,
//! but saved for later evaluation.

use std::collections::{HashMap, HashSet};

use pest::Span;

use crate::phased_parser::grammar;

/// Collect, and subsitute all labels found in the input lines.
/// For EQU labels, expand usages
pub fn expand(input_lines: Vec<String>) -> Vec<String> {
    let lines = collect_and_expand(input_lines);

    // TODO: label substitution

    lines
}

fn collect_and_expand(mut lines: Vec<String>) -> Vec<String> {
    use grammar::Rule;

    let mut collector = Collector::new();

    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];

        let tokenized_line = grammar::tokenize(&line);

        if tokenized_line.is_empty() {
            dbgf!("Empty line: {:?}", line);
            continue;
        }

        dbg!(&tokenized_line);

        let first_token = &tokenized_line[0];

        match first_token.as_rule() {
            Rule::Label => {
                if let Some(next_token) = tokenized_line.get(1) {
                    if next_token.as_rule() == Rule::Substitution {
                        collector.process_equ(first_token.as_str(), next_token.as_str());
                        lines.remove(i);
                        continue;
                    }
                }

                if let Some(LabelValue::Substitution(subst)) =
                    collector.labels.get(first_token.as_str())
                {
                    let expanded = expand_lines(&lines[i], first_token.as_span(), subst);
                    lines.splice(i..=i, expanded.into_iter());
                    continue;
                }

                collector.add_pending_label(first_token.as_str());
                collector.resolve_pending_labels(i);
                // TODO expand all remaining labels in line
            }
            Rule::Substitution => {
                collector.process_equ_continuation(first_token.as_str());
                lines.remove(i);
                continue;
            }
            _ => {
                collector.resolve_pending_labels(i);
                // TODO expand all remaining labels in line
            }
        }

        i += 1;
    }

    collector.finish();

    lines
}

fn expand_lines(line: &str, span: Span, substitution: &[String]) -> Vec<String> {
    assert!(!substitution.is_empty());

    dbgf!("Substituting {:?} for {:?}", substitution, span);

    let before = &line[..span.start()];
    let after = &line[span.end()..];

    let mut lines = substitution.to_vec();

    lines[0] = before.to_owned() + &lines[0];
    lines.last_mut().unwrap().push_str(after);

    lines
}

#[derive(Debug, Eq, PartialEq)]
enum LabelValue {
    // Unresolved, // TODO is this needed for used but undeclared?
    Offset(usize),
    Substitution(Vec<String>),
}

type Labels = HashMap<String, LabelValue>;

#[derive(Debug)]
struct Collector {
    labels: Labels,
    current_equ: Option<(String, Vec<String>)>,
    pending_labels: HashSet<String>,
}

impl Collector {
    fn new() -> Self {
        Self {
            labels: Labels::new(),
            current_equ: None,
            pending_labels: HashSet::new(),
        }
    }

    fn process_equ(&mut self, label: &str, substitution: &str) {
        if substitution.is_empty() {
            // TODO: warning empty RHS of EQU (see docs/pmars-redcode-94.txt:170)
        }

        assert!(
            self.current_equ.is_none(),
            "Already parsing EQU: {:?} but encountered more substitution: {:?}",
            &self.current_equ,
            &substitution,
        );

        self.current_equ = Some((label.to_owned(), vec![substitution.to_owned()]));
    }

    fn process_equ_continuation(&mut self, substitution: &str) {
        if let Some((_, ref mut values)) = self.current_equ {
            values.push(substitution.to_string());
        } else {
            // TODO #25 real error
            eprintln!("Error: first occurrence of EQU without label")
        }
    }

    fn add_pending_label(&mut self, label: &str) {
        self.pending_labels.insert(label.to_owned());
    }

    fn resolve_pending_labels(&mut self, offset: usize) {
        let mut result = HashMap::new();

        let pending_labels = std::mem::replace(&mut self.pending_labels, HashSet::new());
        for pending_label in pending_labels.into_iter() {
            result.insert(pending_label, LabelValue::Offset(offset));
        }

        let current_equ = self.current_equ.take();
        if let Some((multiline_equ_label, values)) = current_equ {
            // Reached the last line in an equ, add to table and reset
            result.insert(multiline_equ_label, LabelValue::Substitution(values));
        }

        self.labels.extend(result.into_iter())
    }

    fn finish(&mut self) {
        if !self.pending_labels.is_empty() {
            // TODO warning for empty definition for each pending label
        }

        self.labels.extend(
            self.current_equ
                .take()
                .map(|(label, values)| (label, LabelValue::Substitution(values))),
        );
    }
}

#[cfg(test)]
mod test {
    use maplit::hashmap;
    use test_case::test_case;

    use super::*;
    use LabelValue::*;

    #[test]
    fn collects_equ() {
        let mut collector = Collector::new();

        collector.process_equ("foo", "1");
        collector.finish();

        assert_eq!(
            collector.labels,
            hashmap! {
                String::from("foo") => Substitution(vec![String::from("1")])
            }
        );
    }

    #[test]
    fn collects_multi_line_equ() {
        let mut collector = Collector::new();

        collector.process_equ("foo", "mov 1, 1");
        collector.process_equ_continuation("jne 0, -1");
        collector.finish();

        assert_eq!(
            collector.labels,
            hashmap! {
                String::from("foo") => Substitution(vec![
                    String::from("mov 1, 1"),
                    String::from("jne 0, -1"),
                ])
            }
        );
    }

    #[test]
    fn collects_label_offset() {
        let mut collector = Collector::new();

        collector.add_pending_label("foo");
        collector.add_pending_label("bar");
        collector.resolve_pending_labels(1);

        collector.add_pending_label("zip");
        collector.add_pending_label("zap");
        collector.add_pending_label("gone");
        collector.finish();

        assert_eq!(
            collector.labels,
            hashmap! {
                String::from("foo") => Offset(1),
                String::from("bar") => Offset(1),
            }
        );
    }

    #[test_case("step", 0, 4, &["a"], &["a"]; "single line")]
    #[test_case("a step", 2, 6, &["a"], &["a a"]; "single line with prefix")]
    #[test_case("step b", 0, 4, &["a"], &["a b"]; "single line with suffix")]
    #[test_case(
        "c step d",
        2,
        6,
        &["a"],
        &["c a d"];
        "single line with prefix and suffix"
    )]
    #[test_case(
        "step",
        0,
        4,
        &["x", "y", "z"],
        &["x", "y", "z"];
        "multi line"
    )]
    #[test_case(
        "a step",
        2,
        6,
        &["x", "y", "z"],
        &["a x", "y", "z"];
        "multi line with prefix"
    )]
    #[test_case(
        "step b",
        0,
        4,
        &["x", "y", "z"],
        &["x", "y", "z b"];
        "multi line with suffix"
    )]
    #[test_case(
        "c step d",
        2,
        6,
        &["x", "y", "z"],
        &["c x", "y", "z d"];
        "multi line with prefix and suffix"
    )]
    fn expands_lines(
        line: &str,
        start: usize,
        end: usize,
        substitution: &[&str],
        expected: &[&str],
    ) {
        let span = Span::new(line, start, end).unwrap();

        let substitution = substitution
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let expanded = expand_lines(line, span, &substitution);

        assert_eq!(expanded, expected);
    }

    #[test]
    fn expands_single_equ() {
        let lines = vec![
            String::from("step equ mov 1, "),
            String::from("nop 1, 1"),
            String::from("step 1"),
            String::from("lbl1"),
            // TODO: the following line tokenized to just "lbl2"
            String::from("lbl2 step 2"),
            String::from("step lbl2"),
            String::from("lbl3 step lbl3"),
        ];

        assert_eq!(
            expand(lines),
            vec![
                String::from("nop 1, 1"),
                String::from("mov 1, 1"),
                String::from("lbl1"),
                String::from("lbl2 mov 1, 2"),
                String::from("mov 1, lbl1"),
                String::from("lbl3 mov 1, lbl3"),
            ]
        );
    }
}
