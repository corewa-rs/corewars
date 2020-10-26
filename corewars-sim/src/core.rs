//! A [`Core`](Core) is a block of "memory" in which Redcode programs reside.
//! This is where all simulation of a Core Wars battle takes place.

// TODO remove this once this lib is hooked up the CLI
#![allow(dead_code)]

use thiserror::Error as ThisError;

use corewars_core::load_file::{self, Instruction, Offset};
use corewars_core::Warrior;

mod address;
mod modifier;
mod opcode;

const DEFAULT_MAXCYCLES: usize = 10_000;

/// An error occurred during loading or core creation
#[derive(ThisError, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// The warrior was longer than the core size
    #[error("warrior has too many instructions to fit in the core")]
    WarriorTooLong,

    /// The specified core size was larger than the allowed max
    #[error("cannot create a core with size {0}; must be less than {}", u32::MAX)]
    InvalidCoreSize(u32),
}

/// An error occurred while executing the core
#[derive(ThisError, Debug, PartialEq)]
#[non_exhaustive]
pub enum ExecutionError {
    /// The warrior attempted to execute a DAT instruction
    #[error("warrior was terminated due to reaching a DAT")]
    ExecuteDat,

    /// The warrior attempted to execute a division by zero
    #[error("warrior was terminated due to division by 0")]
    DivideByZero,

    /// The maximum number of execution steps was reached
    #[error("max number of steps executed")]
    MaxCyclesReached,
}

/// The full memory core at a given point in time
#[derive(Debug)]
pub struct Core {
    instructions: Box<[Instruction]>,
    entry_point: Offset,
    program_counter: Offset,
    steps_taken: usize,
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
            instructions: vec![Instruction::default(); core_size as usize].into_boxed_slice(),
            entry_point: Offset::new(0, core_size),
            program_counter: Offset::new(0, core_size),
            steps_taken: 0,
        })
    }

    /// Clone and returns the next instruction to be executed.
    fn current_instruction(&self) -> Instruction {
        self.get_offset(self.program_counter).clone()
    }

    fn offset<T: Into<i32>>(&self, value: T) -> Offset {
        Offset::new(value.into(), self.size())
    }

    /// Get the number of instructions in the core (available to programs via the `CORESIZE` label)
    pub fn size(&self) -> u32 {
        self.instructions.len() as _
    }

    /// Get an instruction from a given index in the core
    fn get(&self, index: i32) -> &Instruction {
        &self.get_offset(self.offset(index))
    }

    /// Get a mutable instruction from a given index in the core
    fn get_mut(&mut self, index: i32) -> &mut Instruction {
        self.get_offset_mut(self.offset(index))
    }

    /// Get an instruction from a given offset in the core
    fn get_offset(&self, offset: Offset) -> &Instruction {
        &self.instructions[offset.value() as usize]
    }

    /// Get a mutable from a given offset in the core
    fn get_offset_mut(&mut self, offset: Offset) -> &mut Instruction {
        &mut self.instructions[offset.value() as usize]
    }

    /// Write an instruction at a given index into the core
    fn set(&mut self, index: i32, value: Instruction) {
        self.set_offset(self.offset(index), value)
    }

    /// Write an instruction at a given offset into the core
    fn set_offset(&mut self, index: Offset, value: Instruction) {
        self.instructions[index.value() as usize] = value;
    }

    /// Load a [`Warrior`](Warrior) into the core starting at the front (first instruction of the core).
    /// Returns an error if the Warrior was too long to fit in the core, or had unresolved labels
    fn load_warrior(&mut self, warrior: &Warrior) -> Result<(), Error> {
        if warrior.len() > self.size() {
            return Err(Error::WarriorTooLong);
        }

        // TODO check that all instructions are fully resolved? Or require a type
        // safe way of loading a resolved warrior perhaps

        for (i, instruction) in warrior.program.instructions.iter().enumerate() {
            self.instructions[i] = instruction.clone();
        }

        self.entry_point.set_value(match warrior.program.origin {
            Some(entry_point) => entry_point as _,
            None => 0,
        });

        self.program_counter = self.entry_point;

        Ok(())
    }

    /// Run a single cycle of simulation.
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        if self.steps_taken > DEFAULT_MAXCYCLES {
            return Err(ExecutionError::MaxCyclesReached);
        }

        let result = opcode::execute(self)?;

        // If the opcode affected the program counter, avoid incrementing it an extra time
        if let Some(offset) = result.program_counter_offset {
            self.program_counter += offset;
        } else {
            self.program_counter += 1;
        }

        self.steps_taken += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use corewars_core::load_file::{Field, Opcode, Program};

    use super::*;

    /// Create a core from a string. Public since it is used by submodules' tests as well
    pub fn build_core(program: &str) -> Core {
        let warrior = corewars_parser::parse(program).expect("Failed to parse warrior");

        let mut core = Core::new(8000).unwrap();
        core.load_warrior(&warrior).expect("Failed to load warrior");
        core
    }

    #[test]
    fn new_core() {
        let core = Core::new(128).unwrap();
        assert_eq!(core.size(), 128);
    }

    #[test]
    fn load_program() {
        let mut core = Core::new(128).unwrap();

        let warrior = corewars_parser::parse(
            "
            mov $1, #1
            jmp #-1, #2
            jmp #-1, #2
            ",
        )
        .expect("Failed to parse warrior");

        core.load_warrior(&warrior).expect("Failed to load warrior");
        assert_eq!(core.size(), 128);

        assert_eq!(
            &core.instructions[..4],
            &[
                Instruction::new(Opcode::Mov, Field::direct(1), Field::immediate(1)),
                Instruction::new(Opcode::Jmp, Field::immediate(-1), Field::immediate(2)),
                Instruction::new(Opcode::Jmp, Field::immediate(-1), Field::immediate(2)),
                Instruction::default(),
            ]
        );
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

    #[test]
    fn wrap_program_counter_on_overflow() {
        let mut core = build_core("mov $0, $1");

        for i in 0..core.size() {
            assert_eq!(core.program_counter.value(), i);
            core.step().unwrap();
        }

        assert_eq!(core.program_counter.value(), 0);
        core.step().unwrap();
        assert_eq!(core.program_counter.value(), 1);
    }
}
