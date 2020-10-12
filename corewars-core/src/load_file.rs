use std::fmt;

use lazy_static::lazy_static;

mod metadata;
mod program;
mod types;

pub use metadata::Metadata;
pub use program::{Instructions, LabelMap, Program};
pub use types::{AddressMode, Modifier, Offset, Opcode, PseudoOpcode, UOffset, Value};

lazy_static! {
    pub static ref DEFAULT_CONSTANTS: LabelMap = {
        let mut constants = LabelMap::new();
        constants.insert("CORESIZE".into(), 8000);
        constants.insert("MAXPROCESSES".into(), 8000);
        constants.insert("MAXCYCLES".into(), 80_000);
        constants.insert("MAXLENGTH".into(), 100);
        constants.insert("MINDISTANCE".into(), 100);
        constants.insert("ROUNDS".into(), 1);

        // TODO: handle command-line constant redefinition and things like
        // CURLINE, VERSION, WARRIORS, PSPACESIZE
        constants
    };
}

/// The main public struct used to represent a Redcode warrior
#[derive(Debug, Default)]
pub struct Warrior {
    pub program: Program,
    pub metadata: Metadata,
}

impl fmt::Display for Warrior {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.metadata)?;
        write!(formatter, "{}", self.program)
    }
}

impl Warrior {
    /// The number of instrcutions defined in this Warrior's code
    pub fn len(&self) -> u32 {
        self.program.instructions.len() as u32
    }

    /// Whether the warrior's program is empty (i.e. 0 instructions)
    pub fn is_empty(&self) -> bool {
        self.program.instructions.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Field {
    pub address_mode: AddressMode,
    pub value: Value,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&format!("{}{}", self.address_mode, self.value))
    }
}

impl Field {
    pub fn direct(value: Offset) -> Self {
        Self {
            address_mode: AddressMode::Direct,
            value: Value::Literal(value),
        }
    }

    pub fn direct_label<S: ToString>(label: S) -> Self {
        Self {
            address_mode: AddressMode::Direct,
            value: Value::Label(label.to_string()),
        }
    }

    pub fn immediate(value: Offset) -> Self {
        Self {
            address_mode: AddressMode::Immediate,
            value: Value::Literal(value),
        }
    }

    pub fn unwrap_value(&self) -> Offset {
        self.value.unwrap()
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
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&format!(
            // Example output:
            // MOV.AB  $-100,  $1
            // |----->||----->|
            "{op:<8}{a:<8}{b}",
            op = format!("{}.{}", self.opcode, self.modifier),
            a = format!("{},", self.field_a),
            b = self.field_b,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
}
