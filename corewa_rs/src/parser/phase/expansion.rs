//! This phase finds and expands substitutions, namely:
//! - EQU definitions
//! - FOR blocks (not yet implemented)
//! - Standard labels which alias an address
//!
//! Labels used in the right-hand side of an expression substituted in-place.

use std::collections::{HashMap, HashSet};

use pest::Span;

use crate::load_file::{Offset, UOffset};
use crate::parser::grammar;

/// The result of expansion and substitution
#[derive(Debug, Default, PartialEq)]
pub struct Lines {
    pub text: Vec<String>,
    pub origin: Option<String>,
}

/// Collect and subsitute all labels found in the input lines.
pub fn expand(mut text: Vec<String>, mut origin: Option<String>) -> Lines {
    let labels = collect_and_expand(&mut text);

    // TODO: FOR expansion

    substitute_offsets(&mut text, &labels);

    if let Some(mut origin_str) = origin.as_mut() {
        substitute_offsets_in_line(&mut origin_str, &labels, 0);
    }

    Lines { text, origin }
}

/// Collect and strip out offset-based label declarations, meanwhile expanding
/// `EQU` labels.
fn collect_and_expand(lines: &mut Vec<String>) -> Labels {
    use grammar::Rule;

    let mut collector = Collector::new();

    let mut i: usize = 0;
    let mut offset = 0;

    while i < lines.len() {
        // TODO clone
        let line = lines[i].clone();
        let tokenized_line = grammar::tokenize(&line);

        if tokenized_line.is_empty() {
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
            _ => {
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

fn substitute_offsets(lines: &mut Vec<String>, labels: &Labels) {
    let mut i = 0;
    for line in lines.iter_mut() {
        // TODO: clone
        let cloned = line.clone();
        let tokenized_line = grammar::tokenize(&cloned);

        if tokenized_line[0].as_rule() == grammar::Rule::Label {
            if let Some(next_token) = tokenized_line.get(1) {
                line.replace_range(..next_token.as_span().start(), "");
            } else {
                // TODO actually remove line instead
                line.clear();
                // Skip incrementing offset since the line was just a label
                continue;
            }
        }

        substitute_offsets_in_line(line, labels, i);
        i += 1;
    }
}

fn substitute_offsets_in_line(line: &mut String, labels: &Labels, from_offset: UOffset) {
    // TODO: clone
    let cloned = line.clone();
    let tokenized_line = grammar::tokenize(&cloned);

    for token in tokenized_line.iter() {
        if token.as_rule() == grammar::Rule::Label {
            let label_value = labels.get(token.as_str());

            match label_value {
                Some(LabelValue::Offset(offset)) => {
                    let relative_offset = *offset as Offset - from_offset as Offset;
                    let span = token.as_span();
                    line.replace_range(span.start()..span.end(), &relative_offset.to_string());
                }
                _ => {
                    // TODO #25 actual error
                    panic!("No label {:?} found", token.as_str());
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum LabelValue {
    Offset(UOffset),
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

        if self.current_equ.is_some() {
            self.resolve_pending_equ();
        }

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

    fn resolve_pending_labels(&mut self, offset: UOffset) {
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
        &["foo equ 4", "bar equ 1", "mov 1, foo", "nop bar, bar"],
        &["mov 1, 4", "nop 1, 1"];
        "subsequent equ"
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

        assert_eq!(
            expand(lines, None),
            Lines {
                text: expected,
                origin: None,
            }
        );
    }

    #[test_case(
        &[
            "mov 1, 1",
            "start nop 1, 1",
            "mov 2, 3",
        ],
        &[
            "mov 1, 1",
            "nop 1, 1",
            "mov 2, 3",
        ],
        Some(String::from("start")),
        Some(String::from("1"));
        "label"
    )]
    #[test_case(
        &[
            "mov 1, 1",
            "start nop 1, 1",
            "mov 2, 3",
        ],
        &[
            "mov 1, 1",
            "nop 1, 1",
            "mov 2, 3",
        ],
        Some(String::from("start + 1")),
        Some(String::from("1 + 1"));
        "expression"
    )]
    #[test_case(
        &[
            "mov 1, 1",
            "start nop 1, 1",
            "mov 2, 3",
        ],
        &[
            "mov 1, 1",
            "nop 1, 1",
            "mov 2, 3",
        ],
        Some(String::from("1")),
        Some(String::from("1"));
        "literal"
    )]
    #[test_case(
        &[
            "mov 1, 1",
            "start nop 1, 1",
            "mov 2, 3",
        ],
        &[
            "mov 1, 1",
            "nop 1, 1",
            "mov 2, 3",
        ],
        None,
        None;
        "none"
    )]
    fn expands_origin(
        lines: &[&str],
        expected_lines: &[&str],
        origin: Option<String>,
        expected_origin: Option<String>,
    ) {
        let lines = lines.iter().map(|s| s.to_string()).collect();
        let expected: Vec<String> = expected_lines.iter().map(|s| s.to_string()).collect();

        assert_eq!(
            expand(lines, origin),
            Lines {
                text: expected,
                origin: expected_origin,
            }
        );
    }
}
