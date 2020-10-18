//! A [`Core`](Core) is a block of "memory" in which Redcode programs reside.
//! This is where all simulation of a Core Wars battle takes place.

use thiserror::Error as ThisError;

use corewars_core::load_file::{self, Instruction, Offset, Opcode};
use corewars_core::Warrior;

const DEFAULT_MAXCYCLES: usize = 10_000;

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
            instructions: vec![Instruction::default(); core_size as usize].into_boxed_slice(),
            entry_point: Offset::new(0, core_size),
            pointer: Offset::new(0, core_size),
            steps_taken: 0,
        })
    }

    /// Clone and returns the next instruction to be executed.
    fn next_instruction(&self) -> Instruction {
        self.get_offset(self.pointer).clone()
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

        self.pointer = self.entry_point;

        Ok(())
    }

    /// Run a single cycle of simulation.
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        if self.steps_taken > DEFAULT_MAXCYCLES {
            return Err(ExecutionError::MaxCyclesReached);
        }

        let instruction = self.next_instruction();

        // TODO: Use modifier for resolving which part of the instruction gets used
        // for _value see docs/icws94.txt:891 for definitions of {A,B}-{value,target}
        // For now, we just treat everything as I modifier
        let a_field = instruction.a_field;
        let a_pointer = a_field.unwrap_value();
        let a_instruction = self.get(a_pointer);
        let a_value = a_instruction.clone();

        let b_field = instruction.b_field;
        let b_pointer = b_field.unwrap_value();
        let b_instruction = self.get(b_pointer);
        let b_value = b_instruction.clone();

        // let mut write_to_dest = |to_write: WriteInstruction| {
        //     if let Some(opcode) = to_write.opcode {
        //         b_instruction.opcode = opcode
        //     }
        //     if let Some(value) = to_write.a_field {
        //         b_instruction.a_field.set_value(value)
        //     }
        //     if let Some(value) = to_write.b_field {
        //         b_instruction.b_field.set_value(value)
        //     }
        // };

        // See docs/icws94.txt:1113 for detailed description of each opcode
        match instruction.opcode {
            Opcode::Add => self.add(a_value, b_value, b_pointer),
            Opcode::Cmp | Opcode::Seq => {
                if a_value == b_value {
                    self.pointer += 1;
                }
            }
            Opcode::Dat => return Err(ExecutionError::ExecuteDat),
            Opcode::Div => self.div(a_value, b_value, b_pointer)?,
            Opcode::Djn => self.djn(a_value, b_value),
            Opcode::Jmn => todo!(),
            Opcode::Jmp => {
                self.pointer += a_pointer;
                self.steps_taken += 1;
                // Return early to avoid an extra increment of the program counter
                return Ok(());
            }
            Opcode::Jmz => todo!(),
            Opcode::Mod => todo!(),
            Opcode::Mov => self.mov(a_value, b_pointer),
            Opcode::Mul => todo!(),
            Opcode::Nop => todo!(),
            Opcode::Slt => todo!(),
            Opcode::Sne => todo!(),
            Opcode::Spl => todo!(),
            Opcode::Sub => todo!(),
        }

        self.pointer += 1;
        self.steps_taken += 1;
        Ok(())
    }

    fn add(&mut self, lhs: Instruction, rhs: Instruction, dest_index: i32) {
        let a_field = self.offset(lhs.a_field.unwrap_value()) + rhs.a_field.unwrap_value();
        let b_field = self.offset(lhs.b_field.unwrap_value()) + rhs.b_field.unwrap_value();
        let dest = self.get_mut(dest_index);
        dest.a_field.set_value(a_field);
        dest.b_field.set_value(b_field);
    }

    fn div(
        &mut self,
        dividend: Instruction,
        divisor: Instruction,
        dest_index: i32,
    ) -> Result<(), ExecutionError> {
        let a_dividend = self.offset(dividend.a_field.unwrap_value());
        let a_divisor = divisor.a_field.unwrap_value();

        let b_dividend = self.offset(dividend.b_field.unwrap_value());
        let b_divisor = divisor.b_field.unwrap_value();

        let dest = self.get_mut(dest_index);

        if a_divisor != 0 {
            dest.a_field.set_value(a_dividend / a_divisor);
        }
        if b_divisor != 0 {
            dest.b_field.set_value(b_dividend / b_divisor);
        }

        if a_divisor == 0 || b_divisor == 0 {
            Err(ExecutionError::DivideByZero)
        } else {
            Ok(())
        }
    }

    fn djn(&mut self, a_value: Instruction, b_value: Instruction) {
        todo!()
    }

    fn mov(&mut self, value: Instruction, dest_index: i32) {
        let dest = self.get_mut(dest_index);
        dest.opcode = value.opcode;
        dest.a_field = value.a_field;
        dest.b_field = value.b_field;
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use corewars_core::load_file::{Field, Program};

    use super::*;

    fn build_core(opcode: Opcode, a_direct: i32, b_direct: i32) -> Core {
        let mut core = Core::new(4).unwrap();

        let instruction =
            Instruction::new(opcode, Field::direct(a_direct), Field::direct(b_direct));

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
    fn wrap_pointer_on_overflow() {
        let mut core = build_core(Opcode::Mov, 0, 1);

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

    // =========================================================================
    // Begin opcode tests
    // =========================================================================

    #[test]
    fn execute_dat() {
        let mut core = Core::new(4).unwrap();
        // Default instruction is DAT, so expect a termination error immediately
        assert_eq!(core.step().unwrap_err(), ExecutionError::ExecuteDat);
    }

    #[test]
    fn execute_div() {
        let mut core = build_core(Opcode::Div, 1, 2);
        core.set(
            1,
            Instruction::new(Opcode::Dat, Field::direct(8), Field::direct(7)),
        );
        core.set(
            2,
            Instruction::new(Opcode::Dat, Field::direct(4), Field::direct(2)),
        );

        assert!(core.step().is_ok());
        assert_eq!(
            *core.get(2),
            Instruction::new(Opcode::Dat, Field::direct(2), Field::direct(3)),
        )
    }

    #[test_case(
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(2)),
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(3))
            ;
            "a_zero"
        )]
    #[test_case(
            Instruction::new(Opcode::Dat, Field::direct(2), Field::direct(0)),
            Instruction::new(Opcode::Dat, Field::direct(2), Field::direct(0))
            ;
            "b_zero"
        )]
    #[test_case(
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0)),
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0))
            ;
            "both_zero"
        )]
    fn execute_div_by_zero(divisor: Instruction, result: Instruction) {
        use pretty_assertions::assert_eq;

        let mut core = build_core(Opcode::Div, 4, 6);
        core.set(
            1,
            Instruction::new(Opcode::Dat, Field::direct(1), Field::direct(1)),
        );
        core.set(2, divisor);

        assert_eq!(core.step().unwrap_err(), ExecutionError::DivideByZero);
        assert_eq!(*core.get(2), result)
    }

    #[test]
    fn execute_mov() {
        let instruction = Instruction::new(Opcode::Mov, Field::direct(0), Field::direct(1));
        let mut core = build_core(
            instruction.opcode,
            instruction.a_field.unwrap_value(),
            instruction.b_field.unwrap_value(),
        );

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
