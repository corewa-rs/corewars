//! This phase finds and expands substitutions, namely:
//! - EQU definitions
//! - FOR blocks
//! - Standard labels which alias an address
//!
//! Labels used in the right-hand side of an expression are not substituted,
//! but saved for later evaluation.

use std::{collections::HashMap, str::FromStr};

use crate::load_file::types::{Opcode, PseudoOpcode};

pub type Labels = HashMap<String, Option<Vec<String>>>;

#[derive(Debug, Eq, PartialEq)]
pub struct Info {
    pub labels: Labels,
    pub lines: Vec<String>,
}

impl Info {
    pub fn collect(input_lines: Vec<String>) -> Self {
        let mut labels = HashMap::new();
        let mut lines = Vec::new();

        let mut current_equ: Option<(String, Vec<String>)> = None;

        for line in input_lines.into_iter() {
            dbg!(&current_equ);

            let tokenized_line = line.split_whitespace().collect::<Vec<_>>();
            let first_word = tokenized_line[0].to_uppercase();

            if Opcode::from_str(&first_word).is_ok() {
                lines.push(line);

                if let Some((label, values)) = current_equ {
                    // Reached the last line in an equ, insert to table and reset
                    labels.insert(label, Some(values));
                    current_equ = None;
                }
                continue;
            }

            if Self::is_equ(&first_word) {
                if let Some((_, ref mut values)) = &mut current_equ {
                    values.push(tokenized_line[1..].join(" "));
                } else {
                    // TODO (#25) Syntax error, first occurrence of EQU without label
                }
                continue;
            }

            // Assume first_word is a label since it was not a keyword
            if let Some(word) = tokenized_line.get(1) {
                if Self::is_equ(word) {
                    // Begin storing value of this label
                    current_equ = Some((
                        tokenized_line[0].to_owned(),
                        vec![tokenized_line[2..].join(" ")],
                    ));
                }
            }

            if current_equ.is_none() {
                // We encountered a label without EQU, save it for later
                labels.insert(tokenized_line[0].to_owned(), None);
            }
        }

        if let Some((label, values)) = current_equ {
            // Reached the last line of the file, add whatever equ we had
            labels.insert(label, Some(values));
        }

        Self { labels, lines }
    }

    fn is_equ(word: &str) -> bool {
        matches!(
            PseudoOpcode::from_str(&word.to_uppercase()),
            Ok(PseudoOpcode::Equ)
        )
    }
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;

    use super::*;

    #[test]
    fn collect_equ() {
        let lines = vec![String::from("foo equ 1")];

        let info = Info::collect(lines);

        assert_eq!(
            info.labels,
            hashmap! {
                String::from("foo") => Some(vec![String::from("1")]),
            }
        );
        assert_eq!(info.lines, Vec::<String>::new());
    }

    #[test]
    fn collect_multiline_equ() {
        let lines = vec![
            String::from("foo equ mov 1, 1"),
            String::from("equ jne 0, -1"),
        ];

        let info = dbg!(Info::collect(lines));

        assert_eq!(
            info.labels,
            hashmap! {
                String::from("foo") => Some(vec![
                    String::from("mov 1, 1"),
                    String::from("jne 0, -1"),
                ])
            }
        );
        assert_eq!(info.lines, Vec::<String>::new());
    }
}
