use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
    string::ToString,
};

type LabelMap = HashMap<String, usize>;

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
        Self::Direct
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

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub address_mode: AddressMode,
    pub value: Value,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            address_mode: AddressMode::Immediate,
            value: Value::default(),
        }
    }
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

    pub fn resolve(&self, from: usize, labels: &LabelMap) -> Result<Self, String> {
        match &self.value {
            Value::Literal(_) => Ok(self.clone()),
            Value::Label(label) => {
                let label_value = labels
                    .get(label)
                    .ok_or_else(|| format!("Label '{}' not found", &label))?;

                let value = Value::Literal((*label_value as i32) - (from as i32));

                Ok(Self {
                    address_mode: self.address_mode,
                    value,
                })
            }
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub modifier: Modifier,
    pub field_a: Field,
    pub field_b: Field,
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

    pub fn resolve(&self, from: usize, labels: &LabelMap) -> Result<Self, String> {
        let field_a = self.field_a.resolve(from, labels)?;
        let field_b = self.field_b.resolve(from, labels)?;
        Ok(Self {
            opcode: self.opcode,
            modifier: self.modifier,
            field_a,
            field_b,
        })
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
    instructions: Box<[Option<Instruction>]>,
    labels: LabelMap,
}

impl Core {
    pub fn new(core_size: usize) -> Self {
        Core {
            instructions: vec![None; core_size].into_boxed_slice(),
            labels: HashMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<Instruction> {
        self.instructions.get(index)?.clone()
    }

    pub fn get_resolved(&self, index: usize) -> Result<Instruction, String> {
        self.get(index)
            .unwrap_or_default()
            .resolve(index, &self.labels)
    }

    pub fn set(&mut self, index: usize, value: Instruction) {
        self.instructions[index] = Some(value);
    }

    pub fn add_label(&mut self, index: usize, label: String) -> Result<(), String> {
        if index > self.instructions.len() {
            return Err(format!(
                "Address {} is not valid for core of size {}",
                index,
                self.instructions.len()
            ));
        }

        match self.labels.entry(label) {
            Entry::Occupied(entry) => Err(format!("Label '{}' already exists", entry.key())),
            Entry::Vacant(entry) => {
                entry.insert(index);
                Ok(())
            }
        }
    }

    pub fn label_address(&self, label: &str) -> Option<usize> {
        self.labels.get(label).copied()
    }

    pub fn dump(&self) -> String {
        let resolve_result: Result<Vec<_>, String> = self
            .instructions
            .iter()
            .filter(|&maybe_instruction| maybe_instruction.is_some())
            .map(|instruction| instruction.as_ref().unwrap())
            .enumerate()
            .map(|(i, instruction)| instruction.resolve(i, &self.labels))
            .collect();

        match resolve_result {
            Err(msg) => msg,
            Ok(resolved) => resolved.iter().fold(String::new(), |result, instruction| {
                result + &instruction.to_string() + "\n"
            }),
        }
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
            "Labels: {:?}\nCore:\n{}",
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
                address_mode: AddressMode::Immediate,
                value: Value::Literal(0),
            },
            field_b: Field {
                address_mode: AddressMode::Immediate,
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

        core.add_label(123, "baz".into()).expect("Should add baz");
        core.add_label(0, "foo".into()).expect("Should add foo");
        core.add_label(0, "bar".into()).expect("Should add bar");

        core.add_label(256, "goblin".into())
            .expect_err("Should fail to add labels > 200");
        core.add_label(5, "baz".into())
            .expect_err("Should fail to add duplicate label");

        assert_eq!(core.label_address("foo").unwrap(), 0);
        assert_eq!(core.label_address("bar").unwrap(), 0);
        assert_eq!(core.label_address("baz").unwrap(), 123);

        assert!(core.label_address("goblin").is_none());
        assert!(core.label_address("never_mentioned").is_none());
    }

    // TODO: add test for unresolvable label
    #[test]
    fn resolve_labels() {
        let mut core = Core::new(200);

        core.add_label(123, "baz".into()).expect("Should add baz");
        core.add_label(0, "foo".into()).expect("Should add foo");

        core.set(
            5,
            Instruction {
                field_a: Field {
                    value: Value::Label("baz".into()),
                    ..Default::default()
                },
                field_b: Field {
                    value: Value::Label("foo".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let resolved = core.get_resolved(5).expect("Should resolve instruction");
        assert_eq!(
            resolved,
            Instruction {
                field_a: Field {
                    value: Value::Literal(118),
                    ..Default::default()
                },
                field_b: Field {
                    value: Value::Literal(-5),
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    }
}
