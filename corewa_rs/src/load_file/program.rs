//! Definitions for types that hold information about a Redcode warrior (called
//! a Program in memory)

use std::{collections::HashMap, fmt};

use super::{Field, Instruction};

pub type Instructions = Vec<Instruction>;
pub type LabelMap = HashMap<String, usize>;

/// A parsed Redcode program, which can be loaded into a core for execution
#[derive(Default, PartialEq)]
pub struct Program {
    pub instructions: Instructions,
    pub origin: Option<Field>,
}

impl fmt::Debug for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self)
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.instructions
                .iter()
                .fold(String::new(), |result, instruction| {
                    result + &instruction.to_string() + "\n"
                })
        )
    }
}

impl Program {
    pub fn with_capacity(size: usize) -> Self {
        Self {
            instructions: vec![Default::default(); size],
            origin: None,
        }
    }

    pub fn get(&self, index: usize) -> Option<Instruction> {
        self.instructions.get(index).cloned()
    }

    pub fn set(&mut self, index: usize, value: Instruction) {
        if index >= self.instructions.len() {
            self.instructions.resize_with(index + 1, Default::default);
        }

        self.instructions[index] = value;
    }
}
