//! This phase finds and expands substitutions, namely:
//! - EQU definitions
//! - FOR blocks
//! - Standard labels which alias an address
//!
//! Labels used in the right-hand side of an expression are not substituted,
//! but saved for later evaluation.

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use crate::{load_file::types::PseudoOpcode, phased_parser::grammar};

#[derive(Debug, Eq, PartialEq)]
pub enum LabelValue {
    // Unresolved, // is this needed for used but undeclared?
    Offset(usize),
    Substitution(Vec<String>),
}

pub type Labels = HashMap<String, LabelValue>;

#[derive(Debug, Eq, PartialEq)]
pub struct Info {
    pub labels: Labels,
    pub lines: Vec<String>,
}

impl Info {
    pub fn collect_and_expand(lines: Vec<String>) -> Self {
        let mut result = Self {
            labels: HashMap::new(),
            lines: Vec::new(),
        };

        result.collect_label_declarations(lines);

        result
    }

    /// Collect all EQU constructs from the input lines, and
    /// Possible collected label declarations include:
    ///     - <lbl> EQU <text>
    ///     -       EQU <text>
    ///     - <lbl> [instruction]
    fn collect_label_declarations(&mut self, input_lines: Vec<String>) {
        let mut collector = LabelCollector::new();

        for line in input_lines.into_iter() {
            let tokenized_line = grammar::tokenize(&line);
            let first_token = &tokenized_line[0];

            if first_token.as_rule() == grammar::Rule::Substitution {
                collector.process_equ_continuation(first_token.as_str());
                continue;
            }

            if first_token.as_rule() == grammar::Rule::Opcode {
                self.labels
                    .extend(collector.resolve_pending_labels(self.lines.len()));
                self.lines.push(line);
                continue;
            }

            assert!(first_token.as_rule() == grammar::Rule::Label);

            let new_label = first_token.as_str();

            // At this point, treat first_token as a label since it was not a keyword
            if self.labels.contains_key(new_label) || collector.pending_labels.contains(new_label) {
                // TODO substitutions?
                // Definitely want warnings for duplicate label decl
                continue;
            }

            if let Some(second_token) = tokenized_line.get(1) {
                if second_token.as_rule() == grammar::Rule::Substitution {
                    collector.process_equ(new_label, second_token.as_str())
                } else {
                    // Offset label for a standard line
                    collector.pending_labels.insert(new_label.to_owned());
                    self.labels
                        .extend(collector.resolve_pending_labels(self.lines.len()));

                    let line_remainder = &line[second_token.as_span().start()..];
                    self.lines.push(line_remainder.to_owned());
                }
            } else {
                // Label declaration by itself on a line
                collector
                    .pending_labels
                    .insert(first_token.as_str().to_owned());
            }
        }

        self.labels.extend(collector.finish());
    }
}

#[derive(Debug)]
struct LabelCollector {
    pub current_equ: Option<(String, Vec<String>)>,
    pub pending_labels: HashSet<String>,
}

impl LabelCollector {
    fn new() -> Self {
        Self {
            current_equ: None,
            pending_labels: HashSet::new(),
        }
    }

    pub fn process_equ(&mut self, label: &str, substitution: &str) {
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

    pub fn process_equ_continuation(&mut self, substitution: &str) {
        if let Some((_, ref mut values)) = self.current_equ {
            values.push(substitution.to_string());
        } else {
            // TODO (#25) Syntax error, first occurrence of EQU without label
        }
    }

    pub fn resolve_pending_labels(
        &mut self,
        offset: usize,
    ) -> impl Iterator<Item = (String, LabelValue)> {
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

        result.into_iter()
    }

    pub fn finish(self) -> Option<(String, LabelValue)> {
        if !self.pending_labels.is_empty() {
            // TODO warning for empty definition for each pending label
        }

        self.current_equ
            .map(|(label, values)| (label, LabelValue::Substitution(values)))
    }
}

#[cfg(test)]
mod test {
    use maplit::hashmap;

    use super::LabelValue::*;
    use super::*;

    #[test]
    fn collect_equ() {
        let lines = vec![String::from("foo equ 1")];

        let info = Info::collect_and_expand(lines);

        assert_eq!(
            info.labels,
            hashmap! {
                String::from("foo") => Substitution(vec![String::from("1")])
            }
        );
        assert_eq!(info.lines, Vec::<String>::new());
    }

    #[test]
    fn collect_multi_line_equ() {
        let lines = vec![
            String::from("foo equ mov 1, 1"),
            String::from("equ jne 0, -1"),
            String::from("add 2, 3"),
        ];

        let info = Info::collect_and_expand(lines);

        assert_eq!(
            info.labels,
            hashmap! {
                String::from("foo") => Substitution(vec![
                    String::from("mov 1, 1"),
                    String::from("jne 0, -1"),
                ])
            }
        );
        assert_eq!(info.lines, vec![String::from("add 2, 3")]);
    }

    #[test]
    fn collect_label_offset() {
        let lines = vec![String::from("foo mov 1, 1")];

        let info = Info::collect_and_expand(lines);

        assert_eq!(
            info.labels,
            hashmap! {
                String::from("foo") => Offset(0),
            }
        );
        assert_eq!(info.lines, vec![String::from("mov 1, 1")]);
    }

    #[test]
    fn collect_multi_label_offsets() {
        let lines = vec![
            String::from("start"),
            String::from("_alias mov 1, 1"),
            String::from("other_alias"),
            String::from("other_alias2"),
            String::from("MOV 0, 1"),
        ];

        let info = Info::collect_and_expand(lines);

        assert_eq!(
            info.labels,
            hashmap! {
                String::from("start") => Offset(0),
                String::from("_alias") => Offset(0),
                String::from("other_alias") => Offset(1),
                String::from("other_alias2") => Offset(1),
            }
        );
        assert_eq!(
            info.lines,
            vec![String::from("mov 1, 1"), String::from("MOV 0, 1")]
        );
    }
}
