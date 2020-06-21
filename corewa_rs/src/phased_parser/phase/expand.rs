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

use crate::load_file::types::{Opcode, PseudoOpcode};

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
    pub fn collect(lines: Vec<String>) -> Self {
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
            let tokenized_line: Vec<&str> = line.split_whitespace().collect();
            let first_token = tokenized_line[0].to_uppercase();

            if Self::is_equ(&first_token) {
                collector.process_equ_continuation(&tokenized_line);
                continue;
            }

            if Opcode::from_str(&first_token).is_ok() {
                self.labels
                    .extend(collector.resolve_pending_labels(self.lines.len()));
                self.lines.push(line);
                continue;
            }

            // At this point, treat first_token as a label since it was not a keyword
            if self.labels.contains_key(&first_token)
                || collector.pending_labels.contains(&first_token)
            {
                // TODO substitutions?
                // Definitely want warnings for duplicate label decl
                continue;
            }

            if let Some(second_token) = tokenized_line.get(1) {
                if Self::is_equ(second_token) {
                    collector.process_equ(&tokenized_line)
                } else {
                    // label on top of normal line
                    collector
                        .pending_labels
                        .insert(tokenized_line[0].to_string());
                    self.labels
                        .extend(collector.resolve_pending_labels(self.lines.len()));
                    self.lines.push(tokenized_line[1..].join(" "));
                }
            } else {
                // Label declaration by itself on a line
                collector
                    .pending_labels
                    .insert(tokenized_line[0].to_string());
            }
        }

        self.labels.extend(collector.finish());
    }

    fn is_equ(token: &str) -> bool {
        matches!(
            PseudoOpcode::from_str(&token.to_uppercase()),
            Ok(PseudoOpcode::Equ)
        )
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

    pub fn process_equ(&mut self, tokenized_line: &[&str]) {
        assert!(
            self.current_equ.is_none(),
            "Already parsing EQU: {:?} but encountered more in line: {:?}",
            &self.current_equ,
            &tokenized_line,
        );

        if tokenized_line.len() <= 2 {
            // TODO: warning empty RHS of EQU (see docs/pmars-redcode-94.txt:170)
        }

        let substitution = tokenized_line
            .get(2..)
            .filter(|s| !s.is_empty())
            .map(|tokens| vec![tokens.join(" ")])
            .unwrap_or_default();

        self.current_equ = Some((tokenized_line[0].to_owned(), substitution));
    }

    pub fn process_equ_continuation(&mut self, tokenized_line: &[&str]) {
        if let Some((_, ref mut values)) = self.current_equ {
            values.push(tokenized_line[1..].join(" "));
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
mod tests {
    use maplit::hashmap;

    use super::LabelValue::*;
    use super::*;

    #[test]
    fn collect_equ() {
        let lines = vec![String::from("foo equ 1")];

        let info = Info::collect(lines);

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

        let info = Info::collect(lines);

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

        let info = Info::collect(lines);

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

        let info = Info::collect(lines);

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
