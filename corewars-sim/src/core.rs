//! A [`Core`](Core) is a block of "memory" in which Redcode programs reside.
//! This is where all simulation of a Core Wars battle takes place.

use std::cell::Cell;

use thiserror::Error as ThisError;

use corewars_core::load_file::{self, Instruction, Modifier, Offset, Opcode};
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
    fn next_instruction(&self) -> Instruction {
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

    fn perform_opcode<F>(&mut self, instruction: Instruction, field_op: F)
    where
        F: FnMut(Offset, Offset) -> Option<Offset>,
    {
        self.perform_instruction(instruction, field_op, Option::<fn(_, &mut _)>::None)
    }

    fn perform_instruction<F, I>(
        &mut self,
        instruction: Instruction,
        mut field_op: F,
        instruction_op: Option<I>,
    ) where
        F: FnMut(Offset, Offset) -> Option<Offset>,
        I: FnMut(Instruction, &mut Instruction),
    {
        // TODO handle address modes during deref of these "pointer"s
        // For now it's just direct addressing mode

        let a_pointer = instruction.a_field.unwrap_value();
        let a_value = self.get_offset(self.program_counter + a_pointer).clone();
        let a_value_a_offset = self.offset(a_value.a_field.unwrap_value());
        let a_value_b_offset = self.offset(a_value.b_field.unwrap_value());

        let b_pointer = instruction.b_field.unwrap_value();
        let b_value = self.get_offset(self.program_counter + b_pointer).clone();
        let b_value_a_offset = self.offset(b_value.a_field.unwrap_value());
        let b_value_b_offset = self.offset(b_value.b_field.unwrap_value());

        let b_instruction = self.get_mut(b_pointer);

        match instruction.modifier {
            Modifier::A => {
                if let Some(res) = field_op(a_value_a_offset, b_value_a_offset) {
                    b_instruction.a_field.set_value(res);
                }
            }
            Modifier::B => {
                if let Some(res) = field_op(a_value_b_offset, b_value_b_offset) {
                    b_instruction.b_field.set_value(res);
                }
            }
            Modifier::AB => {
                if let Some(res) = field_op(a_value_a_offset, b_value_b_offset) {
                    b_instruction.b_field.set_value(res);
                }
            }
            Modifier::BA => {
                if let Some(res) = field_op(a_value_b_offset, b_value_a_offset) {
                    b_instruction.a_field.set_value(res);
                }
            }
            Modifier::F | Modifier::I => {
                if let Some(a_res) = field_op(a_value_a_offset, b_value_a_offset) {
                    b_instruction.a_field.set_value(a_res);
                }
                if let Some(b_res) = field_op(a_value_b_offset, b_value_b_offset) {
                    b_instruction.b_field.set_value(b_res);
                }

                if instruction.modifier == Modifier::I {
                    if let Some(mut instruction_op) = instruction_op {
                        {
                            instruction_op(instruction, b_instruction);
                        }
                    }
                }
            }
            Modifier::X => {
                if let Some(a_res) = field_op(a_value_b_offset, b_value_a_offset) {
                    b_instruction.a_field.set_value(a_res);
                }
                if let Some(b_res) = field_op(a_value_a_offset, b_value_b_offset) {
                    b_instruction.b_field.set_value(b_res);
                }
            }
        }
    }

    /// Run a single cycle of simulation.
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        if self.steps_taken > DEFAULT_MAXCYCLES {
            return Err(ExecutionError::MaxCyclesReached);
        }

        let instruction = self.next_instruction();
        let zero_offset = self.offset(0);
        let extra_increment = Cell::new(zero_offset);

        // See docs/icws94.txt:1113 for detailed description of each opcode
        match instruction.opcode {
            Opcode::Add => self.perform_opcode(instruction, |a, b| Some(a + b)),
            Opcode::Cmp | Opcode::Seq => {
                extra_increment.set(self.offset(1));
                // For e.g. F and I, all fields must match
                self.perform_instruction(
                    instruction,
                    |a, b| {
                        if a != b {
                            extra_increment.set(zero_offset)
                        }
                        None
                    },
                    Some(|a_instruction, b_instruction: &mut _| {
                        if a_instruction != *b_instruction {
                            extra_increment.set(zero_offset);
                        }
                    }),
                )
            }
            Opcode::Dat => return Err(ExecutionError::ExecuteDat),
            Opcode::Div => {
                let mut div_result = Ok(());
                self.perform_opcode(instruction, |a, b| {
                    if b.value() == 0 {
                        div_result = Err(ExecutionError::DivideByZero);
                        None
                    } else {
                        Some(a / b)
                    }
                });
                div_result?;
            }
            Opcode::Djn => todo!(),
            Opcode::Jmn => todo!(),
            Opcode::Jmp => {
                // Subtract one since we always increment by one anyway
                self.program_counter += instruction.a_field.unwrap_value() - 1;
            }

            Opcode::Jmz => todo!(),
            Opcode::Mod => todo!(),
            Opcode::Mov => self.perform_instruction(
                instruction,
                |a_value, _b_value| Some(a_value),
                Some(
                    |a_instruction: Instruction, b_instruction: &mut Instruction| {
                        b_instruction.modifier = a_instruction.modifier;
                        b_instruction.opcode = a_instruction.opcode;
                    },
                ),
            ),
            Opcode::Mul => todo!(),
            Opcode::Nop => todo!(),
            Opcode::Slt => todo!(),
            Opcode::Sne => todo!(),
            Opcode::Spl => todo!(),
            Opcode::Sub => todo!(),
        }

        self.program_counter += 1;
        self.program_counter += extra_increment.get();
        self.steps_taken += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use corewars_core::load_file::{Field, Program};

    use super::*;

    fn build_core(first_instruction: Instruction) -> Core {
        let mut core = Core::new(12).unwrap();

        let warrior = Warrior {
            program: Program {
                instructions: vec![first_instruction],
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
    fn wrap_program_counter_on_overflow() {
        let mut core = build_core(Instruction::new(
            Opcode::Mov,
            Field::direct(0),
            Field::direct(1),
        ));

        for i in 0..core.size() {
            assert_eq!(core.program_counter.value(), i);
            core.step().unwrap();
        }

        core.step().unwrap();
        assert_eq!(core.program_counter.value(), 0);
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
        let mut core = build_core(Instruction::new(
            Opcode::Div,
            Field::direct(1),
            Field::direct(2),
        ));
        core.set(
            1,
            Instruction::new(Opcode::Dat, Field::direct(8), Field::direct(7)),
        );
        core.set(
            2,
            Instruction::new(Opcode::Dat, Field::direct(4), Field::direct(2)),
        );

        core.step().unwrap();
        assert_eq!(
            *core.get(2),
            Instruction::new(Opcode::Dat, Field::direct(2), Field::direct(3)),
        )
    }

    #[test_case(
        Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(2)),
        Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(3))
        ; "a_zero"
    )]
    #[test_case(
        Instruction::new(Opcode::Dat, Field::direct(2), Field::direct(0)),
        Instruction::new(Opcode::Dat, Field::direct(2), Field::direct(0))
        ; "b_zero"
    )]
    #[test_case(
        Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0)),
        Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0))
        ; "both_zero"
    )]
    fn execute_div_by_zero(divisor: Instruction, result: Instruction) {
        use pretty_assertions::assert_eq;

        let mut core = build_core(Instruction {
            opcode: Opcode::Div,
            modifier: Modifier::F,
            a_field: Field::direct(1),
            b_field: Field::direct(2),
        });

        core.set(
            1,
            Instruction::new(Opcode::Dat, Field::direct(4), Field::direct(6)),
        );
        core.set(2, divisor);

        assert_eq!(core.step().unwrap_err(), ExecutionError::DivideByZero);
        assert_eq!(core.get(2), &result)
    }

    #[test]
    fn execute_mov() {
        // TODO

        let instruction = Instruction {
            opcode: Opcode::Mov,
            modifier: Modifier::I,
            a_field: Field::direct(0),
            b_field: Field::direct(1),
        };
        let mut core = build_core(instruction.clone());

        dbg!(&core);

        core.step().unwrap();
        assert_eq!(core.next_instruction(), instruction);
        core.step().unwrap();
        dbg!(&core);
        assert_eq!(core.next_instruction(), instruction);
    }

    #[test]
    fn execute_jmp() {
        let instruction = Instruction::new(Opcode::Jmp, Field::immediate(3), Field::immediate(0));
        let mut core = build_core(instruction.clone());

        core.step().unwrap();

        assert_eq!(core.program_counter.value(), 3);
        assert_eq!(
            &core.instructions[..4],
            &vec![
                instruction,
                Default::default(),
                Default::default(),
                Default::default()
            ][..]
        );
    }
}
