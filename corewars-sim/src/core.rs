//! A [`Core`](Core) is a block of "memory" in which Redcode programs reside.
//! This is where all simulation of a Core Wars battle takes place.

use std::fmt;

use thiserror::Error as ThisError;

use corewars_core::load_file;
use corewars_core::Warrior;

mod offset;

use offset::Offset;

#[derive(Debug)]
pub struct LoadError;

/// An error occurred during loading or core creation
#[derive(ThisError, Debug)]
pub enum Error {
    /// The warrior was longer than the core size
    #[error("warrior has too many instructions to fit in the core")]
    WarriorTooLong,

    /// The specified core size was larger than the allowed max
    #[error("cannot create a core with size {0}; must be less than {}", u32::MAX)]
    InvalidCoreSize(u32),
}

#[derive(Debug)]
pub struct WarriorTerminated;

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error loading warrior")
    }
}

/// The full memory core at a given point in time
#[derive(Debug)]
pub struct Core {
    instructions: Box<[load_file::Instruction]>,
    entry_point: Offset,
    pointer: Offset,
}

impl Default for Core {
    fn default() -> Self {
        Self::new(load_file::DEFAULT_CONSTANTS["CORESIZE"]).unwrap()
    }
}

impl Core {
    /// Create a new Core with the given number of possible instructions.
    pub fn new(core_size: u32) -> Result<Self, Error> {
        if core_size == u32::MAX {
            return Err(Error::InvalidCoreSize(core_size));
        }

        Ok(Self {
            instructions: vec![load_file::Instruction::default(); core_size as usize]
                .into_boxed_slice(),
            entry_point: Offset::new(0, core_size),
            pointer: Offset::new(0, core_size),
        })
    }

    pub fn current_instruction(&self) -> Offset {
        self.pointer
    }

    /// Get the number of instructions in the core (available to programs via the `CORESIZE` label)
    pub fn size(&self) -> u32 {
        self.instructions.len() as _
    }

    /// Load a [`Warrior`](Warrior) into the core starting at the front (first instruction of the core).
    /// Returns an error if the Warrior was too long to fit in the core, or had unresolved labels
    fn load_warrior(&mut self, warrior: &Warrior) -> Result<(), LoadError> {
        if warrior.len() > self.size() {
            return Err(LoadError);
        }

        // TODO check that all instructions are fully resolved? Or require a type
        // safe way of loading a resolved warrior perhaps

        for (i, instruction) in warrior.program.instructions.iter().enumerate() {
            self.instructions[i] = instruction.clone();
        }

        self.entry_point.set_value(match warrior.program.origin {
            Some(entry_point) => entry_point,
            None => 0,
        });

        self.pointer = self.entry_point;

        Ok(())
    }

    /// Run a single cycle of simulation.
    pub fn step(&mut self) -> Result<(), WarriorTerminated> {
        use load_file::Opcode::*;
        // See docs/icws94.txt:918 for detailed description

        let current_instruction = self.instructions[self.pointer.value() as usize].clone();
        // TODO: Complicated logic for resolving which part of the instruction
        // gets used, see docs/icws94.txt:1025

        match current_instruction.opcode {
            Mov => {
                self.mov(current_instruction.field_a, current_instruction.field_b);
            }
            Dat => return Err(WarriorTerminated),
            Jmp => {
                self.pointer += current_instruction.field_a.unwrap_value();
                // Return early to avoid an extra increment of the program counter
                return Ok(());
            }
            _ => todo!("Opcode not yet implemented"),
        }

        self.pointer += 1;
        // Always normalize the instruction pointer to modulo CORESIZE after an operation
        // TODO safety?

        Ok(())
    }

    fn mov(&mut self, _field_a: load_file::Field, _field_b: load_file::Field) {
        // TODO: for now copy the whole A instruction to the B instruction
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use corewars_core::load_file::{Field, Instruction, Opcode, Program};

    use super::*;

    #[test]
    fn new_core() {
        let core = Core::new(128).unwrap();
        assert_eq!(core.size(), 128);
    }

    #[test]
    fn load_program() {
        let mut core = Core::new(128).unwrap();
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
        let mut core = Core::new(128).unwrap();
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
