//! A [`Core`](Core) is a block of "memory" in which Redcode programs reside.
//! This is where all simulation of a Core Wars battle takes place.

use thiserror::Error as ThisError;

use corewars_core::load_file;
use corewars_core::Warrior;

mod offset;

use offset::Offset;

const DEFAULT_MAX_STEPS: usize = 10_000;

/// An error occurred during loading or core creation
#[derive(ThisError, Debug, PartialEq)]
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
pub enum ExecutionError {
    /// The warrior terminated execution
    #[error("warrior was terminated")]
    Terminated,

    /// The maximum number of execution steps was reached
    #[error("max number of steps executed")]
    StepLimitReached,
}

/// The full memory core at a given point in time
#[derive(Debug)]
pub struct Core {
    instructions: Box<[load_file::Instruction]>,
    entry_point: Offset,
    pointer: Offset,
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
            instructions: vec![load_file::Instruction::default(); core_size as usize]
                .into_boxed_slice(),
            entry_point: Offset::new(0, core_size),
            pointer: Offset::new(0, core_size),
            steps_taken: 0,
        })
    }

    /// Clone and returns the next instruction to be executed.
    pub fn next_instruction(&self) -> load_file::Instruction {
        self.instructions[self.pointer.value() as usize].clone()
    }

    /// Get the number of instructions in the core (available to programs via the `CORESIZE` label)
    pub fn size(&self) -> u32 {
        self.instructions.len() as _
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

        self.pointer = self.entry_point;

        Ok(())
    }

    /// Run a single cycle of simulation.
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        use load_file::Opcode::*;

        if self.steps_taken > DEFAULT_MAX_STEPS {
            return Err(ExecutionError::StepLimitReached);
        }

        let instruction = self.next_instruction();

        // TODO: Use modifier for resolving which part of the instruction gets used, see docs/icws94.txt:1025
        let lhs = instruction.field_a;
        let rhs = instruction.field_b;

        // See docs/icws94.txt:918 for detailed description of each opcode
        match instruction.opcode {
            Mov => {
                self.mov(lhs, rhs);
            }
            Dat => return Err(ExecutionError::Terminated),
            Jmp => {
                self.pointer += lhs.unwrap_value();
                // Return early to avoid an extra increment of the program counter
                return Ok(());
            }
            _ => todo!("Opcode not yet implemented"),
        }

        self.pointer += 1;
        self.steps_taken += 1;
        Ok(())
    }

    fn mov(&mut self, lhs: load_file::Field, rhs: load_file::Field) {
        let src = self.pointer + lhs.unwrap_value();
        let dest = self.pointer + rhs.unwrap_value();
        self.instructions[dest.value() as usize] = self.instructions[src.value() as usize].clone();
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

    #[test]
    fn execute_dat() {
        let mut core = Core::new(4).unwrap();
        // Default instruction is DAT, so expect a termination error immediately
        assert_eq!(core.step().unwrap_err(), ExecutionError::Terminated);
    }

    fn core_with_imp() -> Core {
        let mut core = Core::new(4).unwrap();

        let instruction = Instruction::new(Opcode::Mov, Field::direct(0), Field::direct(1));

        let warrior = Warrior {
            program: Program {
                instructions: vec![instruction],
                origin: None,
            },
            ..Default::default()
        };
        core.load_warrior(&warrior).expect("Failed to load warrior");
        core
    }

    #[test]
    fn wrap_pointer_on_overflow() {
        let mut core = core_with_imp();

        assert_eq!(core.pointer.value(), 0);
        assert!(core.step().is_ok());
        assert_eq!(core.pointer.value(), 1);
        assert!(core.step().is_ok());
        assert_eq!(core.pointer.value(), 2);
        assert!(core.step().is_ok());
        assert_eq!(core.pointer.value(), 3);
        assert!(core.step().is_ok());
        assert_eq!(core.pointer.value(), 0);
    }

    #[test]
    fn execute_mov() {
        let mut core = core_with_imp();
        let instruction = Instruction::new(Opcode::Mov, Field::direct(0), Field::direct(1));

        assert!(core.step().is_ok());
        assert_eq!(core.next_instruction(), instruction);
        assert!(core.step().is_ok());
        assert_eq!(core.next_instruction(), instruction);
    }

    #[test]
    fn execute_jmp() {
        let instruction = Instruction::new(Opcode::Jmp, Field::immediate(2), Field::immediate(0));
        let mut core = Core::new(4).unwrap();
        let warrior = Warrior {
            program: Program {
                instructions: vec![instruction.clone()],
                origin: None,
            },
            ..Default::default()
        };
        core.load_warrior(&warrior).expect("Failed to load warrior");

        assert!(core.step().is_ok());

        assert_eq!(core.pointer.value(), 2);
        assert_eq!(
            &core.instructions[..],
            &vec![
                instruction,
                Default::default(),
                Default::default(),
                Default::default()
            ][..]
        );
    }
}
