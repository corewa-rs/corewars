use std::{collections::HashMap, fmt};

use super::{Instruction, DEFAULT_CONSTANTS};

type Instructions = Vec<Instruction>;
pub type LabelMap = HashMap<String, usize>;

pub struct Program {
    instructions: Instructions,
    resolved: Option<Instructions>,
    labels: LabelMap,
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        (self.resolved.is_some() && self.resolved == other.resolved)
            || self.instructions == other.instructions
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.resolved.is_some() {
            write!(formatter, "{}", self)
        } else {
            write!(formatter, "<unresolved core>")
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self.resolved {
                None => &self.instructions,
                Some(instructions) => instructions,
            }
            .iter()
            .fold(String::new(), |result, instruction| {
                result + &instruction.to_string() + "\n"
            })
        )
    }
}

impl Program {
    pub fn new() -> Self {
        Self {
            instructions: Instructions::new(),
            resolved: None,
            labels: DEFAULT_CONSTANTS.clone(),
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            instructions: vec![Default::default(); size],
            resolved: None,
            labels: DEFAULT_CONSTANTS.clone(),
        }
    }

    pub fn get(&self, index: usize) -> Option<Instruction> {
        match &self.resolved {
            Some(instructions) => &instructions,
            None => &self.instructions,
        }
        .get(index)
        .cloned()
    }

    pub fn set(&mut self, index: usize, value: Instruction) {
        if index >= self.instructions.len() {
            self.instructions.resize_with(index + 1, Default::default);
        }

        self.instructions[index] = value;
    }

    pub fn add_label(&mut self, index: usize, label: String) -> Result<(), String> {
        if index > self.instructions.len() {
            return Err(format!(
                "Address {} is not valid for program of size {}",
                index,
                self.instructions.len()
            ));
        }

        if self.labels.insert(label.clone(), index).is_some() {
            Err(format!("Label '{}' already exists", label))
        } else {
            Ok(())
        }
    }

    pub fn label_address(&self, label: &str) -> Option<usize> {
        self.labels.get(label).copied()
    }

    pub fn resolve(&mut self) -> Result<&mut Self, String> {
        let resolved = self
            .instructions
            .iter()
            .enumerate()
            .map(|(i, instruction)| instruction.resolve(i, &self.labels))
            .collect::<Result<_, String>>()?;

        self.resolved = Some(resolved);

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_file::{Field, Value};

    #[test]
    fn labels() {
        let mut program = Program::with_capacity(200);

        program
            .add_label(123, "baz".into())
            .expect("Should add baz");
        program.add_label(0, "foo".into()).expect("Should add foo");
        program.add_label(0, "bar".into()).expect("Should add bar");

        program
            .add_label(256, "goblin".into())
            .expect_err("Should fail to add labels > 200");
        program
            .add_label(5, "baz".into())
            .expect_err("Should error duplicate label");

        assert_eq!(program.label_address("foo").unwrap(), 0);
        assert_eq!(program.label_address("bar").unwrap(), 0);

        // The _last_ version of a label will be the one we use
        assert_eq!(program.label_address("baz").unwrap(), 5);

        assert!(program.label_address("goblin").is_none());
        assert!(program.label_address("never_mentioned").is_none());
    }

    #[test]
    fn resolve_failure() {
        let mut program = Program::with_capacity(10);

        program.add_label(0, "foo".into()).expect("Should add foo");

        program.set(
            5,
            Instruction {
                field_a: Field {
                    value: Value::Label("not_real".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        program.resolve().expect_err("Should fail to resolve");
    }

    #[test]
    fn resolve_labels() {
        let mut program = Program::with_capacity(10);

        program.add_label(0, "foo".into()).expect("Should add foo");
        program.add_label(7, "baz".into()).expect("Should add baz");

        program.set(
            3,
            Instruction {
                field_a: Field {
                    value: Value::Label("baz".into()),
                    ..Default::default()
                },
                field_b: Field {
                    value: Value::Label("foo".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        program.resolve().expect("Failed to resolve labels");

        assert_eq!(
            program.get(3).expect("Instruction not found"),
            Instruction {
                field_a: Field {
                    value: Value::Literal(4),
                    ..Default::default()
                },
                field_b: Field {
                    value: Value::Literal(-3),
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    }

    #[test]
    fn resolve_constants() {
        let mut program = Program::with_capacity(5);

        program.set(
            0,
            Instruction {
                field_a: Field {
                    value: Value::Label("CORESIZE".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        assert_eq!(
            program.get(0).expect("No instruction found").field_a.value,
            Value::Label("CORESIZE".into())
        );

        program.resolve().expect("Failed to resolve labels");

        let expected_value = *DEFAULT_CONSTANTS.get("CORESIZE").unwrap() as i32;

        assert_eq!(
            program.get(0).expect("No instruction found").field_a.value,
            Value::Literal(expected_value)
        );
    }
}
