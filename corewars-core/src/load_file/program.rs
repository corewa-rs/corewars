//! Definitions for types that hold information about a Redcode warrior (called
//! a Program in memory)

use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

use super::{Instruction, PseudoOpcode};

pub type Instructions = Vec<Instruction>;
pub type LabelMap = HashMap<String, u32>;

/// A parsed Redcode program, which can be loaded into a core for execution
#[derive(Default, PartialEq, Eq)]
pub struct Program {
    /// The list of instructions in the program. These are one-to-one copied into
    /// the core when loaded for execution
    pub instructions: Instructions,

    /// The program's entry point as an instruction index
    pub origin: Option<u32>,
}

impl Program {
    /// The number of instructions defined in this [`Program`]
    #[must_use]
    pub fn len(&self) -> u32 {
        self.instructions
            .len()
            .try_into()
            .expect("Program should not have > u32::MAX instructions")
    }

    /// Whether the warrior's program is empty (i.e. 0 instructions)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    #[must_use]
    pub fn get(&self, index: u32) -> Option<Instruction> {
        self.instructions.get(index as usize).cloned()
    }

    pub fn set(&mut self, index: u32, value: Instruction) {
        let index = index as usize;
        if index >= self.instructions.len() {
            self.instructions.resize_with(index + 1, Default::default);
        }

        self.instructions[index] = value;
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "{{")?;
        writeln!(formatter, "origin: {:?},", self.origin)?;

        let lines = self
            .instructions
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();

        write!(formatter, "lines: {:#?},", lines)?;
        writeln!(formatter, "}}")
    }
}

impl fmt::Display for Program {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = Vec::new();

        // Width to match other instruction types
        lines.push(format!(
            "{:<8}{}",
            PseudoOpcode::Org,
            self.origin.unwrap_or_default()
        ));

        for instruction in &self.instructions {
            lines.push(instruction.to_string());
        }

        write!(formatter, "{}", lines.join("\n"))
    }
}
