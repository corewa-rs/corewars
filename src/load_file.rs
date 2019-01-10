use std::{num::ParseIntError, str::FromStr, vec};

use nom::{digit, line_ending, IResult};

const CORE_SIZE: usize = 8000;

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
    type Err = &'static str;

    fn from_str(input_str: &str) -> Result<Self, Self::Err> {
        use self::Opcode::*;
        match input_str {
            "MOV" => Ok(Mov),
            "DAT" => Ok(Dat),
            _ => Err("Invalid opcode"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum AddressMode {
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

#[derive(Copy, Clone, Debug, PartialEq)]
struct Field {
    value: i32,
    address_mode: AddressMode,
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
    a: Field,
    b: Option<Field>,
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
    instructions: vec::Vec<Instruction>,
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

named!(take_opcode<&str, Opcode>, ws!(
    map_res!(
        alt!(
            tag!("MOV") | tag!("DAT")
        ),
        FromStr::from_str
    ))
);

named!(int32<&str, i32>, ws!(
    map_res!(digit, FromStr::from_str))
);

named!(take_instruction<&str, Instruction>, ws!(
    do_parse!(
        op: take_opcode >>
        field_a: int32 >> tag!(",") >>
        field_b: int32 >>
        (Instruction {
            opcode: op,
            a: Field{value: field_a, ..Default::default()},
            b: Some(Field{value: field_b, ..Default::default()}),
        })
    )
));

named!(take_program<&str, vec::Vec<Instruction>>, many0!(take_instruction));

pub fn parse(file_contents: &str) -> Program {
    let mut program = Program {
        instructions: vec![Instruction::default(); CORE_SIZE],
    };

    // TODO proper error handling
    if let IResult::Done(_, parsed_program) = take_program(file_contents) {
        for (i, &instruction) in parsed_program.iter().enumerate() {
            program.instructions[i] = instruction;
        }
    } else {
        println!("Parser was not Done")
    }

    program
}
