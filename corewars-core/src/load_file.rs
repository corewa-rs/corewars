use std::{convert::TryInto, fmt};

use lazy_static::lazy_static;
use maplit::hashmap;

mod metadata;
mod offset;
mod program;
mod types;

pub use metadata::Metadata;
pub use offset::Offset;
pub use program::{Instructions, LabelMap, Program};
pub use types::{AddressMode, Modifier, Opcode, PseudoOpcode, Value};

lazy_static! {
    // TODO: handle command-line constant redefinition and things like
    // CURLINE, VERSION, WARRIORS, PSPACESIZE
    pub static ref DEFAULT_CONSTANTS: LabelMap = hashmap! {
        "CORESIZE".into() => 8000,
        "MAXPROCESSES".into() => 8000,
        "MAXCYCLES".into() => 80_000,
        "MAXLENGTH".into() => 100,
        "MINDISTANCE".into() => 100,
        "ROUNDS".into() => 1,
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
    /// The number of instructions defined in this Warrior's code
    #[must_use]
    pub fn len(&self) -> usize {
        self.program.instructions.len()
    }

    /// Whether the warrior's program is empty (i.e. 0 instructions)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.program.instructions.is_empty()
    }
}

// It might be useful to have a "target" vs  a "value" function,
// based on the address mode I guess...
//
// See docs/icws94.txt:891
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
    #[must_use]
    pub fn direct(value: i32) -> Self {
        Self {
            address_mode: AddressMode::Direct,
            value: Value::Literal(value),
        }
    }

    pub fn direct_label<S: ToString>(label: &S) -> Self {
        Self {
            address_mode: AddressMode::Direct,
            value: Value::Label(label.to_string()),
        }
    }

    #[must_use]
    pub fn immediate(value: i32) -> Self {
        Self {
            address_mode: AddressMode::Immediate,
            value: Value::Literal(value),
        }
    }

    #[must_use]
    pub fn unwrap_value(&self) -> i32 {
        self.value.unwrap()
    }

    #[must_use]
    pub fn as_offset(&self, core_size: u32) -> Offset {
        Offset::new(self.unwrap_value(), core_size)
    }

    pub fn set_value(&mut self, offset: Offset) {
        self.value = Value::Literal(
            offset
                .value()
                .try_into()
                .expect("Offset should always be convertible to i32"),
        );
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub modifier: Modifier,
    pub a_field: Field,
    pub b_field: Field,
}

impl Instruction {
    #[must_use]
    pub fn new(opcode: Opcode, a_field: Field, b_field: Field) -> Self {
        let modifier =
            Modifier::default_88_to_94(opcode, a_field.address_mode, b_field.address_mode);

        Instruction {
            opcode,
            modifier,
            a_field,
            b_field,
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
            a = format!("{},", self.a_field),
            b = self.b_field,
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
            a_field: Field {
                address_mode: AddressMode::Direct,
                value: Value::Literal(0),
            },
            b_field: Field {
                address_mode: AddressMode::Direct,
                value: Value::Literal(0),
            },
        };

        assert_eq!(Instruction::default(), expected_instruction);
    }
}
