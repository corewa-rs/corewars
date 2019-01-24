use std::{fmt::Debug, str::FromStr, string::ToString};

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AddressMode {
    Immediate,
    Direct,
}

impl AddressMode {
    pub fn to_string(self) -> String {
        use self::AddressMode::*;
        match self {
            Immediate => "#",
            Direct => "",
        }
        .to_owned()
    }
}

impl Default for AddressMode {
    fn default() -> AddressMode {
        AddressMode::Direct
    }
}

impl FromStr for AddressMode {
    type Err = String;

    fn from_str(input_str: &str) -> Result<Self, Self::Err> {
        use self::AddressMode::*;
        match input_str {
            "#" => Ok(Immediate),
            "" => Ok(Direct),
            _ => Err(format!("Invalid address mode '{}'", input_str)),
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Field {
    pub value: i32,
    pub address_mode: AddressMode,
}

impl Field {
    pub fn to_string(self) -> String {
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
    pub fn to_string(&self) -> String {
        format!(
            "{} {}, {}",
            self.opcode.to_string(),
            self.field_a.to_string(),
            self.field_b.to_string(),
        )
    }
}

pub struct Core {
    pub instructions: [Instruction; CORE_SIZE],
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

mod tests {
    use super::*;

    #[test]
    fn default_instruction() {
        assert_eq!(
            Instruction::default(),
            Instruction {
                opcode: Opcode::Dat,
                field_a: Field {
                    value: 0,
                    address_mode: AddressMode::Direct
                },
                field_b: Field {
                    value: 0,
                    address_mode: AddressMode::Direct
                }
            }
        )
    }
}
