use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
    string::ToString,
};

pub const DEFAULT_CORE_SIZE: usize = 8000;

enum_string!(pub Opcode {
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
    fn default() -> Self {
        Opcode::Dat
    }
}

enum_string!(pub PseudoOpcode {
    Org => "ORG",
    Equ => "EQU",
    End => "END",
});

enum_string!(pub Modifier {
    A   => "A",
    B   => "B",
    AB  => "AB",
    BA  => "BA",
    F   => "F",
    X   => "X",
    I   => "I",
});

impl Default for Modifier {
    fn default() -> Self {
        Modifier::F
    }
}

impl Modifier {
    pub fn default_88_to_94(opcode: Opcode, a_mode: AddressMode, b_mode: AddressMode) -> Self {
        /// Implemented based on the ICWS '94 document,
        /// section A.2.1.2: ICWS'88 to ICWS'94 Conversion
        use Opcode::*;

        match opcode {
            Dat => Modifier::F,
            Jmp | Jmz | Jmn | Djn | Spl | Nop => Modifier::B,
            opcode => {
                if a_mode == AddressMode::Immediate {
                    Modifier::AB
                } else if b_mode == AddressMode::Immediate {
                    Modifier::B
                } else {
                    match opcode {
                        Mov | Cmp | Seq | Sne => Modifier::I,
                        Slt => Modifier::B,
                        Add | Sub | Mul | Div | Mod => Modifier::F,
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

enum_string!(pub AddressMode {
    Immediate           => "#",
    Direct              => "$",
    IndirectA           => "*",
    IndirectB           => "@",
    PreDecIndirectA     => "{",
    PreDecIndirectB     => "<",
    PostIncIndirectA    => "}",
    PostIncIndirectB    => ">",
});

impl Default for AddressMode {
    fn default() -> Self {
        Self::Immediate
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Label(String),
    Literal(i32),
}

impl Default for Value {
    fn default() -> Self {
        Self::Literal(0)
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Self::Label(value) => value.clone(),
            Self::Literal(value) => value.to_string(),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Field {
    pub address_mode: AddressMode,
    pub value: Value,
}

impl Field {
    pub fn direct(value: i32) -> Self {
        Self {
            address_mode: AddressMode::Direct,
            value: Value::Literal(value),
        }
    }

    pub fn immediate(value: i32) -> Field {
        Self {
            address_mode: AddressMode::Immediate,
            value: Value::Literal(value),
        }
    }
}

impl ToString for Field {
    fn to_string(&self) -> String {
        format!(
            "{}{}",
            self.address_mode.to_string(),
            self.value.to_string()
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub modifier: Modifier,
    pub field_a: Field,
    pub field_b: Field,
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction {
            opcode: Opcode::default(),
            modifier: Modifier::default(),
            field_a: Field::direct(0),
            field_b: Field::direct(0),
        }
    }
}

impl Instruction {
    pub fn new(opcode: Opcode, field_a: Field, field_b: Field) -> Self {
        let modifier =
            Modifier::default_88_to_94(opcode, field_a.address_mode, field_b.address_mode);

        Instruction {
            opcode,
            modifier,
            field_a,
            field_b,
        }
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        format!(
            "{}.{} {}, {}",
            self.opcode.to_string(),
            self.modifier.to_string(),
            self.field_a.to_string(),
            self.field_b.to_string(),
        )
    }
}

#[derive(PartialEq)]
pub struct Core {
    instructions: Vec<Instruction>,
    labels: HashMap<String, usize>,
}

impl Core {
    pub fn new(core_size: usize) -> Self {
        Core {
            instructions: vec![Instruction::default(); core_size],
            labels: HashMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    pub fn set(&mut self, index: usize, value: Instruction) {
        self.instructions[index] = value;
    }

    pub fn add_labels<L>(&mut self, index: usize, labels: L) -> Result<(), String>
    where
        L: IntoIterator,
        L::Item: Into<String>,
    {
        if index > self.instructions.len() {
            return Err(format!(
                "Address {} is not valid for core of size {}",
                index,
                self.instructions.len()
            ));
        }

        for label in labels {
            match self.labels.entry(label.into()) {
                Entry::Occupied(entry) => {
                    return Err(format!("Label '{}' already exists", entry.key()));
                }
                Entry::Vacant(entry) => {
                    entry.insert(index);
                }
            }
        }

        Ok(())
    }

    pub fn label_address(&self, label: &str) -> Option<usize> {
        self.labels.get(label).copied()
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
    fn default() -> Self {
        Core::new(DEFAULT_CORE_SIZE)
    }
}

impl fmt::Debug for Core {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "Labels:{:?}\nCore:\n{}",
            self.labels,
            self.dump()
        )
    }
}

#[cfg(test)]
mod tests {
    use itertools::iproduct;

    use super::*;
    use Opcode::*;

    #[test]
    fn default_instruction() {
        let expected_instruction = Instruction {
            opcode: Opcode::Dat,
            modifier: Modifier::F,
            field_a: Field {
                address_mode: AddressMode::Direct,
                value: Value::Literal(0),
            },
            field_b: Field {
                address_mode: AddressMode::Direct,
                value: Value::Literal(0),
            },
        };

        assert_eq!(Instruction::default(), expected_instruction)
    }

    #[test]
    fn dat_default() {
        for (&a_mode, &b_mode) in iproduct!(AddressMode::iter_values(), AddressMode::iter_values())
        {
            assert_eq!(
                Modifier::default_88_to_94(Opcode::Dat, a_mode, b_mode),
                Modifier::F
            );
        }
    }

    #[test]
    fn value_to_string() {
        assert_eq!(
            String::from("some_label"),
            Value::Label(String::from("some_label")).to_string()
        );

        assert_eq!(String::from("123"), Value::Literal(123).to_string());
    }

    #[test]
    fn modifier_b_default() {
        let opcodes = [Mov, Cmp, Seq, Sne];

        for (&opcode, &a_mode) in iproduct!(opcodes.iter(), AddressMode::iter_values()) {
            if a_mode != AddressMode::Immediate {
                assert_eq!(
                    Modifier::default_88_to_94(opcode, a_mode, AddressMode::Immediate),
                    Modifier::B
                );
            }
        }

        let opcodes = [Add, Sub, Mul, Div, Mod];

        for (&opcode, &a_mode) in iproduct!(opcodes.iter(), AddressMode::iter_values()) {
            if a_mode != AddressMode::Immediate {
                assert_eq!(
                    Modifier::default_88_to_94(opcode, a_mode, AddressMode::Immediate),
                    Modifier::B
                );
            }
        }

        for (&a_mode, &b_mode) in iproduct!(AddressMode::iter_values(), AddressMode::iter_values())
        {
            if a_mode != AddressMode::Immediate {
                assert_eq!(
                    Modifier::default_88_to_94(Opcode::Slt, a_mode, b_mode),
                    Modifier::B
                )
            }
        }

        let opcodes = [Jmp, Jmz, Jmn, Djn, Spl, Nop];

        for (&opcode, &a_mode, &b_mode) in iproduct!(
            opcodes.iter(),
            AddressMode::iter_values(),
            AddressMode::iter_values()
        ) {
            assert_eq!(
                Modifier::default_88_to_94(opcode, a_mode, b_mode),
                Modifier::B
            );
        }
    }

    #[test]
    fn modifier_ab_default() {
        let opcodes = [Mov, Cmp, Seq, Sne, Add, Sub, Mul, Div, Mod, Slt];

        for (&opcode, &b_mode) in iproduct!(opcodes.iter(), AddressMode::iter_values()) {
            assert_eq!(
                Modifier::default_88_to_94(opcode, AddressMode::Immediate, b_mode),
                Modifier::AB
            );
        }
    }

    #[test]
    fn modifier_i_default() {
        let opcodes = [Mov, Cmp, Seq, Sne];

        for (&opcode, &a_mode, &b_mode) in iproduct!(
            opcodes.iter(),
            AddressMode::iter_values(),
            AddressMode::iter_values()
        ) {
            if a_mode != AddressMode::Immediate && b_mode != AddressMode::Immediate {
                assert_eq!(
                    Modifier::default_88_to_94(opcode, a_mode, b_mode),
                    Modifier::I
                );
            }
        }
    }

    #[test]
    fn modifier_f_default() {
        let opcodes = [Add, Sub, Mul, Div, Mod];

        for (&opcode, &a_mode, &b_mode) in iproduct!(
            opcodes.iter(),
            AddressMode::iter_values(),
            AddressMode::iter_values()
        ) {
            if a_mode != AddressMode::Immediate && b_mode != AddressMode::Immediate {
                assert_eq!(
                    Modifier::default_88_to_94(opcode, a_mode, b_mode),
                    Modifier::F
                );
            }
        }
    }

    #[test]
    fn labels() {
        let mut core = Core::new(200);

        core.add_labels(0, vec!["foo", "bar"]).unwrap();

        core.add_labels(123, vec!["baz", "boo"]).unwrap();

        core.add_labels(256, vec!["goblin"])
            .expect_err("Should have failed to add labels for 256, but didn't");

        core.add_labels(5, vec!["baz"])
            .expect_err("Should have failed to add duplicate label");

        assert_eq!(core.label_address("foo").unwrap(), 0);
        assert_eq!(core.label_address("bar").unwrap(), 0);
        assert_eq!(core.label_address("baz").unwrap(), 123);
        assert_eq!(core.label_address("boo").unwrap(), 123);

        assert!(core.label_address("goblin").is_none());
        assert!(core.label_address("never_mentioned").is_none());
    }
}
