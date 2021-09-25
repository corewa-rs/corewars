//! A [`Core`](Core) is a block of "memory" in which Redcode programs reside.
//! This is where all simulation of a Core Wars battle takes place.

use std::fmt;
use std::ops::{Index, Range};

use thiserror::Error as ThisError;

use corewars_core::load_file::{self, Instruction, Offset};
use corewars_core::Warrior;

mod address;
mod modifier;
mod opcode;
mod process;

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

    #[error(transparent)]
    WarriorAlreadyLoaded(#[from] process::Error),
}

/// The full memory core at a given point in time
pub struct Core {
    instructions: Box<[Instruction]>,
    process_queue: process::Queue,
    steps_taken: usize,
}

impl Core {
    /// Create a new Core with the given number of possible instructions.
    pub fn new(core_size: u32) -> Result<Self, Error> {
        if core_size == u32::MAX {
            return Err(Error::InvalidCoreSize(core_size));
        }

        Ok(Self {
            instructions: vec![Instruction::default(); core_size as usize].into_boxed_slice(),
            process_queue: process::Queue::new(),
            steps_taken: 0,
        })
    }

    #[must_use]
    pub fn steps_taken(&self) -> usize {
        self.steps_taken
    }

    #[cfg(test)]
    fn program_counter(&self) -> Offset {
        self.process_queue
            .peek()
            .expect("process queue was empty")
            .offset
    }

    fn offset<T: Into<i32>>(&self, value: T) -> Offset {
        Offset::new(value.into(), self.size())
    }

    /// Get the number of instructions in the core (available to programs via the `CORESIZE` label)
    #[must_use]
    pub fn size(&self) -> u32 {
        self.instructions.len() as _
    }

    /// Get an instruction from a given index in the core
    #[must_use]
    pub fn get(&self, index: i32) -> &Instruction {
        self.get_offset(self.offset(index))
    }

    /// Get an instruction from a given offset in the core
    fn get_offset(&self, offset: Offset) -> &Instruction {
        &self.instructions[offset.value() as usize]
    }

    /// Get a mutable instruction from a given index in the core
    pub fn get_mut(&mut self, index: i32) -> &mut Instruction {
        self.get_offset_mut(self.offset(index))
    }

    /// Get a mutable from a given offset in the core
    fn get_offset_mut(&mut self, offset: Offset) -> &mut Instruction {
        &mut self.instructions[offset.value() as usize]
    }

    /// Write an instruction at a given index into the core
    #[cfg(test)]
    fn set(&mut self, index: i32, value: Instruction) {
        self.set_offset(self.offset(index), value)
    }

    /// Write an instruction at a given offset into the core
    #[cfg(test)]
    fn set_offset(&mut self, index: Offset, value: Instruction) {
        self.instructions[index.value() as usize] = value;
    }

    /// Load a [`Warrior`](Warrior) into the core starting at the front (first instruction of the core).
    /// Returns an error if the Warrior was too long to fit in the core, or had unresolved labels
    pub fn load_warrior(&mut self, warrior: &Warrior) -> Result<(), Error> {
        if warrior.len() > self.size() {
            return Err(Error::WarriorTooLong);
        }

        // TODO check that all instructions are fully resolved? Or require a type
        // safe way of loading a resolved warrior perhaps

        for (i, instruction) in warrior.program.instructions.iter().enumerate() {
            self.instructions[i] = self.normalize(instruction.clone());
        }

        // TODO: Maybe some kinda increasing counter for warrior names
        let warrior_name = warrior
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| String::from("Warrior0"));

        self.process_queue.push(
            warrior_name,
            self.offset(warrior.program.origin.unwrap_or(0) as i32),
            None,
        );

        Ok(())
    }

    fn normalize(&self, mut instruction: Instruction) -> Instruction {
        // NOTE: this works, but it's a bit unforgiving in terms of debugging since
        // we lose information in the process. Similarly, during parsing we lose
        // expression expansion etc.

        // Maybe it would be better to just normalize all values during execution
        // instead of during warrior loading...

        instruction
            .a_field
            .set_value(self.offset(instruction.a_field.unwrap_value()));

        instruction
            .b_field
            .set_value(self.offset(instruction.b_field.unwrap_value()));

        instruction
    }

    /// Run a single cycle of simulation. This will continue to execute even
    /// after MAXCYCLES has been reached
    pub fn step(&mut self) -> Result<(), process::Error> {
        let current_process = self.process_queue.pop()?;

        eprintln!(
            "Step{:>6} (t{:>2}): {:0>5} {}",
            self.steps_taken,
            current_process.thread,
            current_process.offset.value(),
            self.get_offset(current_process.offset),
        );
        self.steps_taken += 1;

        let result = opcode::execute(self, current_process.offset);

        match result {
            Err(err) => match err {
                process::Error::DivideByZero | process::Error::ExecuteDat(_) => {
                    if self.process_queue.thread_count(&current_process.name) < 1 {
                        Err(err)
                    } else {
                        // This is fine, the task terminated but the process is still alive
                        Ok(())
                    }
                }
                _ => panic!("Unexpected error {}", err),
            },
            Ok(result) => {
                // In the special case of a split, enqueue PC+1 (with same thread id)
                // before also enqueueing the other offset (new thread id)
                let new_thread_id = if result.should_split {
                    self.process_queue.push(
                        current_process.name.clone(),
                        current_process.offset + 1,
                        Some(current_process.thread),
                    );
                    None
                } else {
                    Some(current_process.thread)
                };

                // Either the opcode changed the program counter, or we should just enqueue PC+1
                let offset = result
                    .program_counter_offset
                    .unwrap_or_else(|| self.offset(1));

                self.process_queue.push(
                    current_process.name,
                    current_process.offset + offset,
                    new_thread_id,
                );

                Ok(())
            }
        }
    }

    /// Run a core to completion. Return value determines whether the core resulted
    /// in a tie (Ok) or something cause the warrior to stop executing (ExecutionError)
    pub fn run<T: Into<Option<usize>>>(&mut self, max_cycles: T) -> Result<(), process::Error> {
        let max_cycles = max_cycles.into().unwrap_or(DEFAULT_MAXCYCLES);

        loop {
            if self.steps_taken >= max_cycles {
                break;
            }

            let result = self.step();
            if result.is_err() {
                return result;
            }
        }

        Ok(())
    }

    // TODO: clean up this impl a bunch
    fn format_lines<F: Fn(usize, &Instruction) -> String, G: Fn(usize, &Instruction) -> String>(
        &self,
        formatter: &mut fmt::Formatter,
        instruction_prefix: F,
        instruction_suffix: G,
    ) -> fmt::Result {
        let mut lines = Vec::new();
        let mut iter = self.instructions.iter().enumerate().peekable();

        while let Some((i, instruction)) = iter.next() {
            let add_line = |line_vec: &mut Vec<String>, j| {
                line_vec.push(
                    instruction_prefix(j, instruction)
                        + &instruction.to_string()
                        + &instruction_suffix(j, instruction),
                )
            };

            if *instruction == Instruction::default() {
                // Skip large chunks of defaulted instructions with a counter instead
                let mut skipped_count = 0;
                while let Some(&(_, inst)) = iter.peek() {
                    if inst != &Instruction::default() {
                        break;
                    }
                    skipped_count += 1;
                    iter.next();
                }

                if skipped_count > 5 {
                    add_line(&mut lines, i);
                    lines.push(format!("; {:<6}({} more)", "...", skipped_count - 2));
                    add_line(&mut lines, i + skipped_count);
                } else {
                    for _ in 0..skipped_count {
                        add_line(&mut lines, i);
                    }
                }
            } else {
                add_line(&mut lines, i);
            }
        }

        write!(formatter, "{}", lines.join("\n"))
    }
}

impl Default for Core {
    fn default() -> Self {
        Self::new(load_file::DEFAULT_CONSTANTS["CORESIZE"]).unwrap()
    }
}

impl fmt::Debug for Core {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format_lines(
            formatter,
            |i, _| format!("{:0>6} ", i),
            |i, _| {
                if let Ok(process) = self.process_queue.peek() {
                    if i as u32 == process.offset.value() {
                        return format!("{:>8}", "; <= PC");
                    }
                }

                String::new()
            },
        )
    }
}

impl fmt::Display for Core {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format_lines(formatter, |_, _| String::new(), |_, _| String::new())
    }
}

impl Index<Range<usize>> for Core {
    type Output = [Instruction];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.instructions[index]
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
        let expected_core_size = 128_i32;
        assert_eq!(core.size(), expected_core_size as u32);

        assert_eq!(
            &core.instructions[..4],
            &[
                Instruction::new(Opcode::Mov, Field::direct(1), Field::immediate(1)),
                Instruction::new(
                    Opcode::Jmp,
                    Field::immediate(expected_core_size - 1),
                    Field::immediate(2)
                ),
                Instruction::new(
                    Opcode::Jmp,
                    Field::immediate(expected_core_size - 1),
                    Field::immediate(2)
                ),
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
            assert_eq!(core.program_counter().value(), i);
            core.step().unwrap();
        }

        assert_eq!(core.program_counter().value(), 0);
        core.step().unwrap();
        assert_eq!(core.program_counter().value(), 1);
    }
}
