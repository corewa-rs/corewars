use std::{str::FromStr, vec};

pub const CORE_SIZE: usize = 8000;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Opcode {
    Mov,
    Dat,
}

impl Opcode {
    pub fn to_string(self) -> String {
        use self::Opcode::*;
        match self {
            Mov => "MOV",
            Dat => "DAT",
        }
        .to_owned()
    }
}

impl FromStr for Opcode {
    type Err = String;

    fn from_str(input_str: &str) -> Result<Self, Self::Err> {
        use self::Opcode::*;
        match input_str {
            "MOV" => Ok(Mov),
            "DAT" => Ok(Dat),
            _ => Err(format!("Invalid opcode '{}'", input_str)),
        }
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Field {
    pub value: i32,
    pub address_mode: AddressMode,
}

impl Field {
    pub fn to_string(self) -> String {
        format!("{}{}", self.address_mode.to_string(), self.value)
    }
}

impl Default for Field {
    fn default() -> Field {
        Field {
            value: 0,
            address_mode: AddressMode::Direct,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub a: Field,
    pub b: Option<Field>,
}

impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {
            opcode: Opcode::Dat,
            a: Field::default(),
            b: Some(Field::default()),
        }
    }
}

impl Instruction {
    pub fn to_string(&self) -> String {
        format!(
            "{} {}{}",
            self.opcode.to_string(),
            self.a.to_string(),
            match &self.b {
                Some(field) => format!(", {}", field.to_string()),
                None => "".to_owned(),
            }
        )
    }
}

#[derive(Debug)]
pub struct Program {
    pub instructions: vec::Vec<Instruction>,
}

impl Program {
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
