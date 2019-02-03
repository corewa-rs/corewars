use std::{fmt, string::ToString, vec};

pub const DEFAULT_CORE_SIZE: usize = 8000;

enum_string!(pub Opcode, {
    Dat => "DAT",
    Mov => "MOV",
    Add => "ADD",
    Sub => "SUB",
    Mul => "MUL",
    Div => "DIV",
    Mod => "MOD",
    Jmp => "JMP",
    Jmz => "JMZ",
    Jmn => "JMN",
    Djn => "DJN",
    Cmp => "CMP",
    Seq => "SEQ",
    Sne => "SNE",
    Slt => "SLT",
    Spl => "SPL",
    Nop => "NOP",
});

impl Default for Opcode {
    fn default() -> Opcode {
        Opcode::Dat
    }
}

enum_string!(pub PseudoOpcode, {
    Org => "ORG",
    Equ => "EQU",
    End => "END",
});

enum_string!(pub AddressMode, {
    Immediate => "#",
    Direct => "$",
    IndirectA => "*",
    IndirectB => "@",
    PreDecIndirectA => "{",
    PreDecIndirectB => "<",
    PostIncIndirectA => "}",
    PostIncIndirectB => ">",
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
    instructions: vec::Vec<Instruction>,
}

impl Core {
    pub fn new(core_size: usize) -> Core {
        Core {
            instructions: vec![Instruction::default(); core_size],
        }
    }

    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    pub fn set(&mut self, index: usize, value: Instruction) {
        self.instructions[index] = value;
    }

    pub fn dump_all(&self) -> String {
        self.instructions
            .iter()
            .fold(String::new(), |result, instruction| {
                result + &instruction.to_string() + "\n"
            })
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
        Core::new(DEFAULT_CORE_SIZE)
    }
}

impl fmt::Debug for Core {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.dump_all())
    }
}

impl PartialEq for Core {
    fn eq(&self, rhs: &Self) -> bool {
        for (self_instruction, other_instruction) in
            self.instructions.iter().zip(rhs.instructions.iter())
        {
            if self_instruction != other_instruction {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
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
