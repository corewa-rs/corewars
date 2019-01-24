use std::{fmt::Debug, string::ToString};

pub const CORE_SIZE: usize = 8000;

enum_string!(pub Opcode, {
    Mov => "MOV",
    Dat => "DAT",
    Jmp => "JMP",
});

impl Default for Opcode {
    fn default() -> Opcode {
        Opcode::Dat
    }
}

enum_string!(pub AddressMode, {
    Immediate => "#",
    Direct => "",
});

impl Default for AddressMode {
    fn default() -> AddressMode {
        AddressMode::Direct
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Field {
    pub address_mode: AddressMode,
    pub value: i32,
}

impl Field {
    pub fn direct(value: i32) -> Field {
        Field {
            address_mode: AddressMode::Direct,
            value,
        }
    }

    pub fn immediate(value: i32) -> Field {
        Field {
            address_mode: AddressMode::Immediate,
            value,
        }
    }
}

impl ToString for Field {
    fn to_string(&self) -> String {
        format!("{}{}", self.address_mode.to_string(), self.value)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub field_a: Field,
    pub field_b: Field,
}

impl Instruction {
    pub fn new(opcode: Opcode, field_a: Field, field_b: Field) -> Instruction {
        Instruction {
            opcode,
            field_a,
            field_b,
        }
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        format!(
            "{} {}, {}",
            self.opcode.to_string(),
            self.field_a.to_string(),
            self.field_b.to_string(),
        )
    }
}

pub struct Core {
    instructions: [Instruction; CORE_SIZE],
}

impl Core {
    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    pub fn set(&mut self, index: usize, value: Instruction) {
        self.instructions[index] = value;
    }

    pub fn dump(&self) -> String {
        self.instructions
            .iter()
            .filter(|&instruction| *instruction != Instruction::default())
            .fold(String::new(), |result, instruction| {
                result + &instruction.to_string() + "\n"
            })
    }
}

impl Default for Core {
    fn default() -> Core {
        Core {
            instructions: [Instruction::default(); CORE_SIZE],
        }
    }
}

impl Debug for Core {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.dump())
    }
}

impl PartialEq for Core {
    fn eq(&self, rhs: &Self) -> bool {
        for (i, instruction) in self.instructions.iter().enumerate() {
            if let Some(other_instruction) = rhs.get(i) {
                if other_instruction != instruction {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

mod tests {
    use super::*;

    #[test]
    fn default_instruction() {
        let expected_instruction = Instruction {
            opcode: Opcode::Dat,
            field_a: Field {
                address_mode: AddressMode::Direct,
                value: 0,
            },
            field_b: Field {
                address_mode: AddressMode::Direct,
                value: 0,
            },
        };

        assert_eq!(Instruction::default(), expected_instruction)
    }
}
