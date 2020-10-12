//! A [`Core`](Core) is a block of "memory" in which Redcode programs reside.
//! This is where all simulation of a Core Wars battle takes place.

use std::fmt;

use corewars_core::load_file::{Instruction, UOffset, DEFAULT_CONSTANTS};
use corewars_core::Warrior;

#[derive(Debug)]
struct LoadError;

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error loading warrior")
    }
}

/// The full memory core at a given point in time
#[derive(Debug)]
pub struct Core {
    pub instructions: Box<[Instruction]>,
    pub entry_point: UOffset,
}

impl Default for Core {
    fn default() -> Self {
        Self::new(DEFAULT_CONSTANTS["CORESIZE"])
    }
}

impl Core {
    pub fn new(core_size: usize) -> Self {
        Self {
            instructions: vec![Instruction::default(); core_size].into_boxed_slice(),
            entry_point: 0,
        }
    }

    /// Get the number of instructions in the core (available to programs via the `CORESIZE` label)
    pub fn size(&self) -> usize {
        self.instructions.len()
    }

    fn load_warrior(&mut self, warrior: &Warrior) -> Result<(), LoadError> {
        if warrior.len() > self.size() {
            return Err(LoadError);
        }

        // TODO check that all instructions are fully resolved?

        for (i, instruction) in warrior.program.instructions.iter().enumerate() {
            self.instructions[i] = instruction.clone();
        }

        self.entry_point = match warrior.program.origin {
            Some(entry_point) => entry_point,
            None => 0,
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use corewars_core::load_file::{Field, Opcode, Program};

    use super::*;

    #[test]
    fn new_core() {
        let core = Core::new(128);
        assert_eq!(core.size(), 128);
    }

    #[test]
    fn load_program() {
        let mut core = Core::new(128);
        let instructions = vec![
            Instruction::new(Opcode::Mov, Field::direct(1), Field::immediate(1)),
            Instruction::new(Opcode::Jmp, Field::immediate(-1), Field::immediate(2)),
            Instruction::new(Opcode::Jmp, Field::immediate(-1), Field::immediate(2)),
        ];

        let warrior = Warrior {
            program: Program {
                instructions: instructions.clone(),
                origin: None,
            },
            ..Default::default()
        };

        core.load_warrior(&warrior).expect("Failed to load warrior");
        assert_eq!(core.size(), 128);

        assert_eq!(&core.instructions[..instructions.len()], &instructions[..]);
    }

    #[test]
    fn load_program_too_long() {
        let mut core = Core::new(128);
        let warrior = Warrior {
            program: Program {
                instructions: vec![
                    Instruction::new(Opcode::Dat, Field::direct(1), Field::direct(1),);
                    255
                ],
                origin: None,
            },
            ..Default::default()
        };

        core.load_warrior(&warrior)
            .expect_err("Should have failed to load warrior: too long");

        assert_eq!(core.size(), 128);
    }
}
