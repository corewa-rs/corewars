use std::fmt;

enum_string!(pub Opcode {
    Add => "ADD",
    Cmp => "CMP",
    Dat => "DAT",
    Div => "DIV",
    Djn => "DJN",
    Jmn => "JMN",
    Jmp => "JMP",
    Jmz => "JMZ",
    Mod => "MOD",
    Mov => "MOV",
    Mul => "MUL",
    Nop => "NOP",
    Seq => "SEQ",
    Slt => "SLT",
    Sne => "SNE",
    Spl => "SPL",
    Sub => "SUB",
});

enum_string!(pub PseudoOpcode {
    Org => "ORG",
    End => "END",
    Equ => "EQU",
    For => "FOR",
});

impl Default for Opcode {
    fn default() -> Self {
        Self::Dat
    }
}

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
        Self::F
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

impl Value {
    pub fn unwrap(&self) -> i32 {
        match *self {
            Value::Literal(i32) => i32,
            _ => panic!("unwrapped value of a Value without a literal i32"),
        }
    }
}

impl From<i32> for Value {
    fn from(i32: i32) -> Self {
        Self::Literal(i32)
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Literal(0)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value_string = match self {
            Self::Label(value) => value.to_string(),
            Self::Literal(value) => value.to_string(),
        };
        f.pad(&value_string)
    }
}

#[cfg(test)]
mod test {
    use itertools::iproduct;

    use super::*;
    use Opcode::*;

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
    fn value_to_string() {
        assert_eq!(
            String::from("some_label"),
            Value::Label(String::from("some_label")).to_string()
        );

        assert_eq!(String::from("123"), Value::Literal(123).to_string());
    }
}
