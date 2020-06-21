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
    fn collect_label_declarations(&mut self, input_lines: Vec<String>) {
        use LabelValue::*;

        let mut current_equ: Option<(String, Vec<String>)> = None;
        let mut pending_labels: HashSet<String> = HashSet::new();

        for line in input_lines.into_iter() {
            let tokenized_line: Vec<&str> = line.split_whitespace().collect();
            let first_token = tokenized_line[0].to_uppercase();

            if Self::is_equ(&first_token) {
                if let Some((_, ref mut values)) = &mut current_equ {
                    values.push(tokenized_line[1..].join(" "));
                } else {
                    // TODO (#25) Syntax error, first occurrence of EQU without label
                }
                continue;
            }

            if Opcode::from_str(&first_token).is_ok() {
                self.lines.push(line);

                for pending_label in pending_labels.into_iter() {
                    self.labels.insert(pending_label, Offset(self.lines.len()));
                }
                pending_labels = HashSet::new();

                if let Some((multiline_equ_label, values)) = current_equ {
                    // Reached the last line in an equ, add to table and reset
                    self.labels
                        .insert(multiline_equ_label, Substitution(values));
                    current_equ = None;
                }
                continue;
            }

            // At this point, treat first_token as a label since it was not a keyword

            if self.labels.contains_key(&first_token) {
                // TODO substitutions, warnings, etc.
                continue;
            }

            if let Some(second_token) = tokenized_line.get(1) {
                if Self::is_equ(second_token) {
                    assert!(
                        current_equ.is_none(),
                        "Already parsing EQU: {:?} but encountered more in line: {:?}",
                        &current_equ,
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

                    current_equ = Some((tokenized_line[0].to_owned(), substitution));
                } else {
                    // label on top of normal line
                    // TODO dedupe with `Opcode::from_str(&first_token).is_ok()` above
                    self.lines.push(tokenized_line[1..].join(" "));

                    pending_labels.insert(first_token);
                    for pending_label in pending_labels.into_iter() {
                        self.labels.insert(pending_label, Offset(self.lines.len()));
                    }
                    pending_labels = HashSet::new();

                    if let Some((multiline_equ_label, values)) = current_equ {
                        // Reached the last line in an equ, add to table and reset
                        self.labels
                            .insert(multiline_equ_label, Substitution(values));
                        current_equ = None;
                    }
                }
            } else {
                // Label declaration by itself on a line
                pending_labels.insert(first_token);
                continue;
            }
        }

        // TODO dedupe with `Opcode::from_str(&first_token).is_ok()` above

        if !pending_labels.is_empty() {
            // TODO warning for empty definition for each pending label
        }

        if let Some((multiline_equ_label, values)) = current_equ {
            // Reached EOF while in EQU, add to table
            self.labels
                .insert(multiline_equ_label, Substitution(values));
        }
    }

    fn is_equ(token: &str) -> bool {
        matches!(
            PseudoOpcode::from_str(&token.to_uppercase()),
            Ok(PseudoOpcode::Equ)
        )
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
    fn collect_multiline_equ() {
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
}
