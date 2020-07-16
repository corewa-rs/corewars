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

/// Collect and subsitute all labels found in the input lines.
pub fn expand(mut lines: Vec<String>) -> Vec<String> {
    let labels = collect_and_expand(&mut lines);

    // TODO: FOR expansion

    dbg!(&labels);

    substitute_offsets(&mut lines, labels);

    lines
}

/// Collect and strip out offset-based label declarations, meanwhile expanding
/// `EQU` labels.
fn collect_and_expand(lines: &mut Vec<String>) -> Labels {
    use grammar::Rule;

    let mut collector = Collector::new();

    let mut i = 0;
    let mut offset = i;
    while i < lines.len() {
        // TODO clone
        let line = lines[i].clone();
        let tokenized_line = grammar::tokenize(&line);

        if tokenized_line.is_empty() {
            dbgf!("Line {} is empty", i);
            continue;
        }

        let first_token = &tokenized_line[0];

        // Returns true if anything was expanded, false otherwise
        let mut expand_next_token = |collector: &Collector| {
            for token in tokenized_line[1..].iter() {
                if token.as_rule() == Rule::Label {
                    if let Some(LabelValue::Substitution(subst)) =
                        collector.labels.get(token.as_str())
                    {
                        dbgf!("Expanded next token in line {}: {:?}", i, lines[i]);
                        expand_lines(lines, i, token.as_span(), subst);
                        return true;
                    }
                }
            }

            false
        };

        match first_token.as_rule() {
            Rule::Label => {
                if let Some(next_token) = tokenized_line.get(1) {
                    if next_token.as_rule() == Rule::Substitution {
                        collector.process_equ(first_token.as_str(), next_token.as_str());
                        lines.remove(i);
                        continue;
                    }
                }

                collector.resolve_pending_equ();

                if let Some(LabelValue::Substitution(substitution)) =
                    collector.labels.get(first_token.as_str())
                {
                    dbgf!("Expanding substitution into line {}: {:?}", i, lines[i]);
                    expand_lines(lines, i, first_token.as_span(), substitution);
                    continue;
                }

                collector.add_pending_label(first_token.as_str());

                if expand_next_token(&collector) {
                    continue;
                }

                if tokenized_line.len() > 1 {
                    collector.resolve_pending_labels(offset);
                    offset += 1;

                    let next_token = tokenized_line[1].as_span();
                    lines[i] = line[next_token.start()..].to_owned();
                } else {
                    lines.remove(i);
                    continue;
                }
            }
            Rule::Substitution => {
                collector.process_equ_continuation(first_token.as_str());
                lines.remove(i);
                continue;
            }
            other => {
                dbgf!(
                    "Encountered {:?} rule in line {}: {:?}, pending labels: {:?}, offset {}",
                    other,
                    i,
                    line,
                    &collector.pending_labels,
                    offset
                );

                collector.resolve_pending_labels(offset);

                if expand_next_token(&collector) {
                    continue;
                }

                if tokenized_line.len() > 1 {
                    collector.resolve_pending_labels(offset);
                    offset += 1;
                }
            }
        }

        i += 1;
    }

    collector.finish()
}

fn expand_lines(lines: &mut Vec<String>, index: usize, span: Span, substitution: &[String]) {
    // TODO clone
    let line = &lines[index];

    let before = &line[..span.start()];
    let after = &line[span.end()..];

    assert!(!substitution.is_empty());
    let mut new_lines = substitution.to_vec();

    new_lines[0] = before.to_owned() + &new_lines[0];
    new_lines.last_mut().unwrap().push_str(after);

    lines.splice(index..=index, new_lines);
}

fn substitute_offsets(lines: &mut Vec<String>, labels: Labels) {
    let mut i = 0;
    for line in lines.iter_mut() {
        {
            // TODO: clone
            let cloned = line.clone();
            let tokenized_line = grammar::tokenize(&cloned);

            dbg!(&tokenized_line);

            if tokenized_line[0].as_rule() == grammar::Rule::Label {
                if let Some(next_token) = tokenized_line.get(1) {
                    line.replace_range(..next_token.as_span().start(), "");
                } else {
                    dbgf!("Line {} is empty", i);
                    // TODO actually remove line instead
                    line.clear();
                    // Skip incrementing offset since the line was just a label
                    continue;
                }
            }
        }

        // TODO: clone
        let cloned = line.clone();
        let tokenized_line = grammar::tokenize(&cloned);

        dbgf!("Line is {:?}, current offset is {:?}", line, i);

        for token in tokenized_line.iter() {
            if token.as_rule() == grammar::Rule::Label {
                let label_value = labels.get(token.as_str());

                match label_value {
                    Some(LabelValue::Offset(offset)) => {
                        let relative_offset = dbg!(*offset as i32) - dbg!(i as i32);
                        let span = token.as_span();
                        line.replace_range(
                            span.start()..span.end(),
                            &dbg!(relative_offset).to_string(),
                        );
                    }
                    _ => {
                        // TODO #25 actual error
                        panic!("No label {:?} found", token.as_str());
                    }
                }
            }
        }

        dbg!(&line);
        i += 1;
    }
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

        let pending_labels = std::mem::take(&mut self.pending_labels);
        for pending_label in pending_labels.into_iter() {
            result.insert(pending_label, LabelValue::Offset(offset));
        }

        self.resolve_pending_equ();

        self.labels.extend(result.into_iter())
    }

    fn resolve_pending_equ(&mut self) {
        let current_equ = self.current_equ.take();

        if let Some((multiline_equ_label, values)) = current_equ {
            // Reached the last line in an equ, add to table and reset
            self.labels
                .insert(multiline_equ_label, LabelValue::Substitution(values));
        }
    }

    fn finish(mut self) -> Labels {
        if !self.pending_labels.is_empty() {
            // TODO warning for empty definition for each pending label
        }

        self.labels.extend(
            self.current_equ
                .take()
                .map(|(label, values)| (label, LabelValue::Substitution(values))),
        );

        self.labels
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
        let labels = collector.finish();

        assert_eq!(
            labels,
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
        let labels = collector.finish();

        assert_eq!(
            labels,
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
        let labels = collector.finish();

        assert_eq!(
            labels,
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

        let mut lines = vec![line.to_string()];

        expand_lines(&mut lines, 0, span, &substitution);

        assert_eq!(lines, expected);
    }

    #[test_case(
        &[
            "lbl1",
            "mov 1, 1",
        ],
        hashmap!{
            "lbl1".into() => LabelValue::Offset(0),
        };
        "single label"
    )]
    #[test_case(
        &[
            "lbl1 mov 1, 1",
        ],
        hashmap!{
            "lbl1".into() => LabelValue::Offset(0),
        };
        "single label statement"
    )]
    #[test_case(
        &[
            "lbl1",
            "lbl2 mov 1, 1",
        ],
        hashmap!{
            "lbl1".into() => LabelValue::Offset(0),
            "lbl2".into() => LabelValue::Offset(0),
        };
        "label alias"
    )]
    #[test_case(
        &[
            "nop 1, 1",
            "lbl1",
            "lbl2 mov 1, 2",
            "mov 2, 3",
            "lbl3 mov 3, 4",
        ],
        hashmap!{
            "lbl1".into() => LabelValue::Offset(1),
            "lbl2".into() => LabelValue::Offset(1),
            "lbl3".into() => LabelValue::Offset(3),
        };
        "multiple labels"
    )]
    #[test_case(
        &[
            "foo equ 1",
            "nop 1, foo",
            "lbl1",
            "lbl2 mov 1, foo",
            "mov 2, foo",
            "lbl3 mov 3, foo",
        ],
        hashmap!{
            "foo".into() => LabelValue::Substitution(vec!["1".into()]),
            "lbl1".into() => LabelValue::Offset(1),
            "lbl2".into() => LabelValue::Offset(1),
            "lbl3".into() => LabelValue::Offset(3),
        };
        "label with expansion"
    )]
    fn collects_offset_label(lines: &[&str], expected: Labels) {
        let mut lines = lines.iter().map(|s| s.to_string()).collect();
        assert_eq!(collect_and_expand(&mut lines), expected);
    }

    #[test_case(
        &["step equ 4", "mov 1, step"],
        &["mov 1, 4"];
        "expression equ"
    )]
    #[test_case(
        &[
            "step equ mov 1, 2",
            "lbl1 step",
            "step",
            "nop lbl1, 0"],
        &[
            "mov 1, 2",
            "mov 1, 2",
            "nop -2, 0",
        ];
        "statement equ"
    )]
    #[test_case(
        &[
            "step equ mov 1,",
            "step lbl1",
            "lbl1",
            "lbl2 step 2",
            "step lbl2",
            "lbl3 step lbl3",
        ],
        &[
            "mov 1, 1",
            "mov 1, 2",
            "mov 1, -1",
            "mov 1, 0",
        ];
        "partial statement equ"
    )]
    #[test_case(
        &[
            "do_thing equ mov 1, 2",
            "equ mov 3, 4",
            "do_thing",
            "lbl1 do_thing",
            "nop 0, lbl1",
        ],
        &[
            "mov 1, 2",
            "mov 3, 4",
            "mov 1, 2",
            "mov 3, 4",
            "nop 0, -2",
        ];
        "multiline equ"
    )]
    fn expands_substitutions(lines: &[&str], expected: &[&str]) {
        let lines = lines.iter().map(|s| s.to_string()).collect();
        let expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();

        assert_eq!(expand(lines), expected);
    }
}
