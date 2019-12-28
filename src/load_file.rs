use std::fmt;

use lazy_static::lazy_static;

mod program;
mod types;

pub use program::{LabelMap, Program};
pub use types::{AddressMode, Modifier, Opcode, Value};

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

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub address_mode: AddressMode,
    pub value: Value,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            address_mode: AddressMode::Immediate,
            value: Default::default(),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&format!("{}{}", self.address_mode, self.value))
    }
}

impl Field {
    pub fn direct(value: i32) -> Self {
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

    pub fn immediate(value: i32) -> Self {
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
                    value,
                    ..self.clone()
                })
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub modifier: Modifier,
    pub field_a: Field,
    pub field_b: Field,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&format!(
            "{}.{:<2} {}, {}",
            self.opcode, self.modifier, self.field_a, self.field_b,
        ))
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

    pub fn resolve(&self, from: usize, labels: &LabelMap) -> Result<Self, String> {
        let field_a = self.field_a.resolve(from, labels)?;
        let field_b = self.field_b.resolve(from, labels)?;
        Ok(Self {
            field_a,
            field_b,
            ..self.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
