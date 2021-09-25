//! This phase finds and expands substitutions, namely:
//! - EQU definitions
//! - FOR blocks (not yet implemented)
//! - Standard labels which alias an address
//!
//! Labels used in the right-hand side of an expression substituted in-place.

use std::collections::{HashMap, HashSet};
use std::string::ToString;

use pest::Span;

use crate::grammar;

use super::evaluation;

use corewars_core::load_file::DEFAULT_CONSTANTS;

/// The result of expansion and substitution
#[derive(Debug, Default, PartialEq)]
pub struct Lines {
    pub text: Vec<String>,
    pub origin: Option<String>,
}

/// Collect and subsitute all labels found in the input lines.
pub fn expand(mut text: Vec<String>, mut origin: Option<String>) -> Lines {
    let labels = collect_and_expand(&mut text);

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
    let mut offset: u32 = 0;

    while i < lines.len() {
        let line = lines[i].clone();
        let tokenized_line = grammar::tokenize(&line);

        if tokenized_line.is_empty() {
            continue;
        }

        let first_token = &tokenized_line[0];

        // Returns true if anything was expanded, false otherwise
        let mut expand_next_token = |collector: &Collector, is_for_expr: bool| {
            for token in tokenized_line[1..].iter() {
                if token.as_rule() == Rule::Label {
                    let label_value = collector.get_label_value(token.as_str(), offset);

                    if let Some(label_value) = label_value {
                        match label_value {
                            LabelValue::AbsoluteOffset(abs_offset) => {
                                let relative_offset = (abs_offset as i32) - (offset as i32);
                                expand_lines(
                                    lines,
                                    i,
                                    token.as_span(),
                                    &[relative_offset.to_string()],
                                );
                            }
                            LabelValue::RelativeOffset(rel_offset) => {
                                expand_lines(lines, i, token.as_span(), &[rel_offset.to_string()]);
                            }
                            LabelValue::Substitution(subst) => {
                                expand_lines(lines, i, token.as_span(), &subst);
                            }
                        }

                        return true;
                    }

                    if is_for_expr {
                        panic!("No label value found for expr {:?}", token.as_str())
                    } else {
                        // this is probably a forward usage of a label not
                        // yet declared, which _could_ be an error
                    }
                }
            }

            false
        };

        match first_token.as_rule() {
            Rule::For => {
                collector.resolve_pending_labels(offset);

                if expand_next_token(&collector, true) {
                    continue;
                }

                let line_remainder = &line[first_token.as_span().end()..];
                collector.push_for(None, i, offset, line_remainder);
                // Continue processing lines as normal, since we still need to collect
                // labels and potentially nested for loops
            }
            Rule::Rof => {
                let for_stmt = collector.pop_for();

                let range_to_repeat = (for_stmt.start_line + 1)..i;
                let insert_line_count = for_stmt.iter_count as usize * range_to_repeat.len();

                // We need to subtract the offset, since we end up replacing
                // those lines. They will be processed normally after substitution
                offset -= range_to_repeat.len() as u32;

                let new_contents = lines[range_to_repeat.clone()]
                    .iter()
                    .cloned()
                    .cycle()
                    .take(insert_line_count)
                    .enumerate()
                    .map(|(i, mut line)| {
                        if let Some(label) = &for_stmt.index_label {
                            let subst = i / range_to_repeat.len() + 1;

                            // This method of label stuff is a little hacky,
                            // but seems reasonable I guess...
                            collector
                                .labels
                                .insert(label.clone(), LabelValue::RelativeOffset(subst as i32));

                            substitute_offsets_in_line(
                                &mut line,
                                &collector.labels,
                                for_stmt.start_offset + (i as u32),
                            );

                            collector.labels.remove(label);
                        }
                        line
                    })
                    .collect::<Vec<_>>();

                let range_to_replace = for_stmt.start_line..=i;
                lines.splice(range_to_replace, new_contents);

                i = for_stmt.start_line;

                continue;
            }
            Rule::Label => {
                if let Some(next_token) = tokenized_line.get(1) {
                    match next_token.as_rule() {
                        Rule::Substitution => {
                            collector.process_equ(first_token.as_str(), next_token.as_str());
                            lines.remove(i);
                            continue;
                        }
                        Rule::For => {
                            collector.resolve_pending_labels(offset);

                            if !expand_next_token(&collector, true) {
                                let line_remainder = &line[next_token.as_span().end()..];

                                collector.push_for(
                                    first_token.as_str().to_string(),
                                    i,
                                    offset,
                                    line_remainder,
                                );

                                i += 1;
                            }
                            continue;
                        }
                        _ => {}
                    }
                }

                collector.resolve_pending_equ();

                if let Some(LabelValue::Substitution(substitution)) =
                    collector.get_label_value(first_token.as_str(), offset)
                {
                    expand_lines(lines, i, first_token.as_span(), &substitution);
                    continue;
                }

                collector.add_pending_label(first_token.as_str());

                if expand_next_token(&collector, false) {
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
            other_rule => {
                collector.resolve_pending_labels(offset);

                if expand_next_token(&collector, false) {
                    continue;
                }

                if tokenized_line.len() > 1 {
                    collector.resolve_pending_labels(offset);

                    if other_rule != Rule::Opcode || first_token.as_str().to_uppercase() != "ORG" {
                        offset += 1;
                    }
                }
            }
        }

        i += 1;
    }

    collector.finish()
}

fn expand_lines(lines: &mut Vec<String>, index: usize, span: Span, substitution: &[String]) {
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
        let cloned = line.clone();
        let tokenized_line = grammar::tokenize(&cloned);

        if tokenized_line[0].as_rule() == grammar::Rule::Label {
            if let Some(next_token) = tokenized_line.get(1) {
                line.replace_range(..next_token.as_span().start(), "");
            } else {
                line.clear();
                // Skip incrementing offset since the line was just a label
                continue;
            }
        }

        substitute_offsets_in_line(line, labels, i);

        if tokenized_line[0].as_rule() != grammar::Rule::Opcode
            || tokenized_line[0].as_str().to_uppercase() != "ORG"
        {
            i += 1;
        }
    }
}

fn substitute_offsets_in_line(line: &mut String, labels: &Labels, from_offset: u32) {
    let tokenized_line = grammar::tokenize(line);

    for token in &tokenized_line {
        if token.as_rule() == grammar::Rule::Label {
            let label_value = labels.get(token.as_str());

            let relative_offset = match label_value {
                Some(&LabelValue::AbsoluteOffset(offset)) => (offset as i32) - (from_offset as i32),
                Some(&LabelValue::RelativeOffset(offset)) => offset,
                _ => continue,
            };

            let span = token.as_span();

            let range = span.start()..span.end();
            let replace_with = relative_offset.to_string();
            line.replace_range(range, &replace_with);

            // Recursively re-parse line and continue substitution.
            // This is less efficient, but means we don't need to deal
            // with the fact that the whole line was invalidate after
            // `replace_range`
            return substitute_offsets_in_line(line, labels, from_offset);
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum LabelValue {
    AbsoluteOffset(u32),
    RelativeOffset(i32),
    Substitution(Vec<String>),
}

type Labels = HashMap<String, LabelValue>;

fn default_labels() -> Labels {
    DEFAULT_CONSTANTS
        .iter()
        // Counterintuitively, we use a relative offset here so that it doesn't
        // get translated like absolute offset labels would be
        .map(|(lbl, value)| (lbl.clone(), LabelValue::RelativeOffset(*value as i32)))
        .collect()
}

#[derive(Debug)]
struct ForStatement {
    index_label: Option<String>,
    iter_count: u32,
    start_line: usize,
    start_offset: u32,
}

#[derive(Debug)]
struct Collector {
    labels: Labels,
    current_equ: Option<(String, Vec<String>)>,
    pending_labels: HashSet<String>,
    for_stack: Vec<ForStatement>,
    for_offsets: HashMap<String, u32>,
}

impl Collector {
    fn new() -> Self {
        Self {
            labels: default_labels(),
            current_equ: None,
            pending_labels: HashSet::new(),
            for_stack: Vec::new(),
            for_offsets: HashMap::new(),
        }
    }

    fn process_equ(&mut self, label: &str, substitution: &str) {
        if substitution.is_empty() {
            // TODO #25 warning empty RHS of EQU (see docs/pmars-redcode-94.txt:170)
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

    fn resolve_pending_labels(&mut self, offset: u32) {
        let mut result = HashMap::new();

        let pending_labels = std::mem::take(&mut self.pending_labels);
        for pending_label in pending_labels {
            result.insert(pending_label, LabelValue::AbsoluteOffset(offset));
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

    fn push_for<T: Into<Option<String>>>(
        &mut self,
        label: T,
        line: usize,
        offset: u32,
        expression: &str,
    ) {
        // TODO handle errors instead of unwrap
        let expr_value = evaluation::evaluate_expression(expression.to_string()).unwrap();

        self.for_stack.push(ForStatement {
            index_label: label.into(),
            iter_count: expr_value,
            start_line: line,
            start_offset: offset,
        });
    }

    fn pop_for(&mut self) -> ForStatement {
        let for_stmt = self.for_stack.pop().unwrap();
        if let Some(label) = &for_stmt.index_label {
            self.for_offsets
                .insert(label.clone(), for_stmt.start_offset);
        }
        for_stmt
    }

    fn get_label_value(&self, label: &str, current_offset: u32) -> Option<LabelValue> {
        let value = self.labels.get(label).cloned();

        if let Some(LabelValue::Substitution(_)) = value {
            // Always return substitutions even if we are in a for loop
            value
        } else if self.for_stack.is_empty() {
            // Otherwise, only expand labels if we are finished unrolling loops
            // This ensures the relative offsets are calculated after the unroll
            if value.is_some() {
                value
            } else if label == "CURLINE" {
                // Special-case for current line number
                Some(LabelValue::RelativeOffset(current_offset as i32))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn finish(mut self) -> Labels {
        if !self.pending_labels.is_empty() {
            // TODO #25 warning for empty definition for each pending label
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
    use LabelValue::{AbsoluteOffset, Substitution};

    #[test]
    fn collects_equ() {
        let mut collector = Collector::new();

        collector.process_equ("foo", "1");
        let labels = collector.finish();

        assert_eq!(
            Some(&Substitution(vec![String::from("1")])),
            labels.get("foo"),
        );
    }

    #[test]
    fn collects_multi_line_equ() {
        let mut collector = Collector::new();

        collector.process_equ("foo", "mov 1, 1");
        collector.process_equ_continuation("jne 0, -1");
        let labels = collector.finish();

        assert_eq!(
            Some(&Substitution(vec![
                String::from("mov 1, 1"),
                String::from("jne 0, -1"),
            ])),
            labels.get("foo")
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

        assert_eq!(Some(&AbsoluteOffset(1)), labels.get("foo"),);
        assert_eq!(Some(&AbsoluteOffset(1)), labels.get("bar"),);
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
            .map(ToString::to_string)
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
            "lbl1".into() => LabelValue::AbsoluteOffset(0),
        };
        "single label"
    )]
    #[test_case(
        &[
            "lbl1 mov 1, 1",
        ],
        hashmap!{
            "lbl1".into() => LabelValue::AbsoluteOffset(0),
        };
        "single label statement"
    )]
    #[test_case(
        &[
            "lbl1",
            "lbl2 mov 1, 1",
        ],
        hashmap!{
            "lbl1".into() => LabelValue::AbsoluteOffset(0),
            "lbl2".into() => LabelValue::AbsoluteOffset(0),
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
            "lbl1".into() => LabelValue::AbsoluteOffset(1),
            "lbl2".into() => LabelValue::AbsoluteOffset(1),
            "lbl3".into() => LabelValue::AbsoluteOffset(3),
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
            "lbl1".into() => LabelValue::AbsoluteOffset(1),
            "lbl2".into() => LabelValue::AbsoluteOffset(1),
            "lbl3".into() => LabelValue::AbsoluteOffset(3),
        };
        "label with expansion"
    )]
    fn collects_and_expands_labels(lines: &[&str], expected: Labels) {
        let mut lines = lines.iter().map(ToString::to_string).collect();
        let result = collect_and_expand(&mut lines);

        for (k, v) in &expected {
            assert_eq!(Some(v), result.get(k));
        }
    }

    #[test_case(
        &[
            "for 3",
            "mov 0, 1",
            "rof",
        ],
        &[
            "mov 0, 1",
            "mov 0, 1",
            "mov 0, 1",
        ];
        "repeat"
    )]
    #[test_case(
        &[
            "for 0",
            "mov 0, 1",
            "rof",
        ],
        &[];
        "empty"
    )]
    #[test_case(
        &[
            "for 3",
            "mov 0, 1",
            "mov 2, 3",
            "rof",
        ],
        &[
            "mov 0, 1",
            "mov 2, 3",
            "mov 0, 1",
            "mov 2, 3",
            "mov 0, 1",
            "mov 2, 3",
        ];
        "repeat multiple"
    )]
    #[test_case(
        &[
            "for 2",
            "for 3",
            "mov 0, 1",
            "rof",
            "mov 1, 2",
            "rof",
        ],
        &[
            "mov 0, 1",
            "mov 0, 1",
            "mov 0, 1",
            "mov 1, 2",
            "mov 0, 1",
            "mov 0, 1",
            "mov 0, 1",
            "mov 1, 2",
        ];
        "nested for"
    )]
    #[test_case(
        &[
            "for 2 + 2",
            "mov 0, 1",
            "rof",
        ],
        &[
            "mov 0, 1",
            "mov 0, 1",
            "mov 0, 1",
            "mov 0, 1",
        ];
        "evaluate expression"
    )]
    #[test_case(
        &[
            "base",
            "N for 2 + 2",
            "mov base, N",
            "rof",
        ],
        &[
            "mov 0, 1",
            "mov -1, 2",
            "mov -2, 3",
            "mov -3, 4",
        ];
        "repeat index"
    )]
    #[test_case(
        &[
            "base",
            "N for 3",
            "mov base, N",
            "mov base, N",
            "rof",
        ],
        &[
            "mov 0, 1",
            "mov -1, 1",
            "mov -2, 2",
            "mov -3, 2",
            "mov -4, 3",
            "mov -5, 3",
        ];
        "repeat index twice"
    )]
    #[test_case(
        &[
            "foo equ mov 0, 1",
            "for 3",
            "foo",
            "rof",
        ],
        &[
            "mov 0, 1",
            "mov 0, 1",
            "mov 0, 1",
        ];
        "repeat and expand"
    )]
    #[test_case(
        &[
            "for 3",
            "foo equ mov 0, 1",
            "rof",
            "foo",
        ],
        &[
            "mov 0, 1",
        ];
        "repeat equ"
    )]
    #[test_case(
        &[
            "five equ 5",
            "lbl mov 0, 1",
            "dat 0, 0",
            "for five + lbl", // 5 - 2
            "mov 1, 2",
            "rof",
        ],
        &[
            "mov 0, 1",
            "dat 0, 0",
            "mov 1, 2",
            "mov 1, 2",
            "mov 1, 2",
        ];
        "expand expr labels"
    )]
    fn collects_and_expands_forrof(lines: &[&str], expected: &[&str]) {
        let mut lines = lines.iter().map(ToString::to_string).collect();
        let _ = collect_and_expand(&mut lines);

        let expected_lines: Vec<String> = expected.iter().map(ToString::to_string).collect();

        assert_eq!(lines, expected_lines);
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
            "nop lbl1, 0",
        ],
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
    #[test_case(
        &[
            "org lbl_b",
            "four equ 2+2",
            "lbl_a dat four+1, four+3", // lbl_a = 0
            "nop -1, -1",
            "lbl_b dat 1, 1",   // lbl_b = 2
            "nop -1, -1",
            "lbl_c add #lbl_a+1, #lbl_b+four+3+4",
        ],
        &[
            "org 2",
            "dat 2+2+1, 2+2+3",
            "nop -1, -1",
            "dat 1, 1",
            "nop -1, -1",
            "add #-4+1, #-2+2+2+3+4",
        ];
        "equ in expression"
    )]
    #[test_case(
        &[
            "add 2, CURLINE",
            "mov 1, CORESIZE-1",
            "add 3, CURLINE",
            "dat MAXLENGTH, MAXLENGTH",
        ],
        &[
            "add 2, 0",
            "mov 1, 8000-1",
            "add 3, 2",
            "dat 100, 100",
        ];
        "expand default labels"
    )]
    fn expands_substitutions(lines: &[&str], expected: &[&str]) {
        let lines = lines.iter().map(ToString::to_string).collect();
        let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();

        assert_eq!(
            Lines {
                text: expected,
                origin: None,
            },
            expand(lines, None),
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
        let lines = lines.iter().map(ToString::to_string).collect();
        let expected: Vec<String> = expected_lines.iter().map(ToString::to_string).collect();

        assert_eq!(
            expand(lines, origin),
            Lines {
                text: expected,
                origin: expected_origin,
            }
        );
    }
}
