//! This phase finds and expands substitutions, namely:
//! - EQU definitions
//! - FOR blocks
//! - Standard labels which alias an address
//!
//! Labels used in the right-hand side of an expression are not substituted,
//! but saved for later evaluation.

use std::collections::{HashMap, HashSet};

use crate::phased_parser::grammar;

/// Collect, and subsitute all labels found in the input lines.
/// For EQU labels, expand usages
pub fn expand(lines: Vec<String>) -> Vec<String> {
    todo!()
}

#[derive(Debug, Eq, PartialEq)]
pub enum LabelValue {
    // Unresolved, // TODO is this needed for used but undeclared?
    Offset(usize),
    Substitution(Vec<String>),
}

pub type Labels = HashMap<String, LabelValue>;

#[derive(Debug, Eq, PartialEq)]
struct Collector {}

impl Info {
    fn collect_and_expand(lines: Vec<String>) -> Self {
        let mut result = Self {
            labels: HashMap::new(),
            lines: Vec::new(),
        };

        result.collect_label_declarations(lines);

        dbg!(&result.lines);
        dbg!(&result.labels);

        result.expand_equ_usages();

        dbg!(&result.lines);

        // TODO offset substitution

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
            if tokenized_line.is_empty() {
                dbgf!("Empty line: {:?}", line);
                continue;
            }

            let first_token = &tokenized_line[0];

            match first_token.as_rule() {
                grammar::Rule::Substitution => {
                    collector.process_equ_continuation(first_token.as_str());
                }
                grammar::Rule::Opcode => {
                    self.labels
                        .extend(collector.resolve_pending_labels(self.lines.len()));
                    self.lines.push(line);
                }
                grammar::Rule::Label => {
                    let found_label = dbg!(first_token.as_str());

                    if let Some(value) = self.labels.get(found_label) {
                        match value {
                            LabelValue::Substitution(_) => {
                                // Usage of a predeclared label
                                self.lines.push(line);
                            }
                            LabelValue::Offset(_) => {
                                // TODO real warning
                                eprintln!(
                                    "Duplicate label {:?}, second declaration will be ignored",
                                    found_label
                                );
                            }
                        }
                    } else if let Some(second_token) = tokenized_line.get(1) {
                        if second_token.as_rule() == grammar::Rule::Substitution {
                            dbgf!("Found EQU: {:?}", second_token.as_str());
                            collector.process_equ(found_label, second_token.as_str())
                        } else {
                            // Offset label for a standard line
                            collector.pending_labels.insert(found_label.to_owned());
                            self.labels
                                .extend(collector.resolve_pending_labels(self.lines.len()));

                            let line_remainder = &line[second_token.as_span().start()..];
                            self.lines.push(line_remainder.to_owned());
                        }
                    } else {
                        if !self.labels.contains_key(first_token.as_str()) {
                            // Offset-based label declaration
                            collector
                                .pending_labels
                                .insert(first_token.as_str().to_owned());
                        }

                        for token in tokenized_line[1..].iter() {
                            if token.as_rule() == grammar::Rule::Label {
                                // Could be a usage of a previously declared label
                            }
                        }

                        self.lines.push(line);
                    }
                }
                rule => panic!("Unexpected token of type {:?}: {:?}", rule, first_token),
            }
        }

        self.labels.extend(collector.finish());
    }

    /// Expand usages of all declared labels. For now, this only expands EQU
    /// labels, but it should eventually handle FOR expansion as well
    fn expand_equ_usages(&mut self) {
        use LabelValue::*;

        let mut i = 0;
        while dbg!(i) < self.lines.len() {
            // TODO: clone
            let current_line = self.lines[i].clone();
            let tokenized = grammar::tokenize(&current_line);

            dbg!(&current_line);
            dbg!(&tokenized);
            dbg!(&self.labels);

            for token in tokenized.iter() {
                if token.as_rule() == grammar::Rule::Label {
                    dbgf!("Found label {:?}", token.as_str());

                    match self.labels.get(token.as_str()) {
                        Some(Offset(_)) => {
                            // TODO
                            dbgf!("Type: offset");
                        }
                        Some(Substitution(subst_lines)) => {
                            assert!(!subst_lines.is_empty());

                            dbgf!("Type: substitution = {:?}", subst_lines);

                            let span = token.as_span();

                            let before = &current_line[..span.start()];
                            let after = &current_line[span.end()..];

                            self.lines[i] = before.to_owned() + &subst_lines[0];

                            if subst_lines.len() > 1 {
                                for line_to_insert in &subst_lines[1..subst_lines.len() - 1] {
                                    // TODO clone
                                    self.lines.insert(i, line_to_insert.clone());
                                    i += 1;
                                }
                            }

                            self.lines[i].push_str(after);
                        }
                        None => {
                            // TODO #25 real error
                            // panic!("No such label {:?}", token.as_str())
                        }
                    }
                }
            }

            i += 1;
        }
    }
}

#[derive(Debug)]
struct LabelCollector {
    labels: Labels,
    current_equ: Option<(String, Vec<String>)>,
    pending_labels: HashSet<String>,
}

impl LabelCollector {
    fn new() -> Self {
        Self {
            labels: Labels::new(),
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
            // TODO #25 real error
            eprintln!("Error: first occurrence of EQU without label")
        }
    }

    pub fn resolve_pending_labels(&mut self, offset: usize) {
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

        dbg!(&result);

        self.labels.extend(result.into_iter())
    }

    pub fn finish(&mut self) {
        if !self.pending_labels.is_empty() {
            // TODO warning for empty definition for each pending label
        }

        self.labels.extend(
            self.current_equ
                .map(|(label, values)| (label, LabelValue::Substitution(values))),
        );
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

    #[test]
    fn expand_single_equ() {
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

        let info = Info::collect_and_expand(lines);

        assert_eq!(
            info.lines,
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
