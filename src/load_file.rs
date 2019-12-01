use std::{collections::HashMap, fmt};

mod types;
pub use types::{AddressMode, Modifier, Opcode, Value};

pub const DEFAULT_CORE_SIZE: usize = 8000;

type Instructions = Box<[Option<Instruction>]>;
type LabelMap = HashMap<String, usize>;

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub address_mode: AddressMode,
    pub value: Value,
}

impl Default for Field {
    fn default() -> Self { Self {
            address_mode: AddressMode::Immediate,
            value: Default::default(),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.address_mode, self.value)
    }
}

impl Field {
    pub fn direct(value: i32) -> Self {
        Self {
            address_mode: AddressMode::Direct,
            value: Value::Literal(value),
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
        write!(
            f,
            "{}.{} {}, {}",
            self.opcode, self.modifier, self.field_a, self.field_b,
        )
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

pub struct Core {
    instructions: Instructions,
    resolved: Option<Instructions>,
    labels: LabelMap,
}

impl PartialEq for Core {
    fn eq(&self, other: &Self) -> bool {
        (self.resolved.is_some() && self.resolved == other.resolved)
            || self.instructions == other.instructions
    }
}

impl Default for Core {
    fn default() -> Self {
        Core::new(DEFAULT_CORE_SIZE)
    }
}

impl fmt::Debug for Core {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.resolved.is_some() {
            write!(formatter, "{}", self)
        } else {
            write!(formatter, "<unresolved core>")
        }
    }
}

impl Core {
    pub fn new(core_size: usize) -> Self {
        Core {
            instructions: vec![None; core_size].into_boxed_slice(),
            resolved: None,
            labels: LabelMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<Instruction> {
        match &self.resolved {
            Some(instructions) => instructions,
            None => &self.instructions,
        }
        .get(index)?
        .clone()
    }

    pub fn set(&mut self, index: usize, value: Instruction) {
        self.instructions[index] = Some(value);
        // TODO need to re-resolve here? Or make caller do it
    }

    pub fn add_label(&mut self, index: usize, label: String) -> Result<(), String> {
        if index > self.instructions.len() {
            return Err(format!(
                "Address {} is not valid for core of size {}",
                index,
                self.instructions.len()
            ));
        }

        if self.labels.insert(label.clone(), index).is_some() {
            Err(format!("Label '{}' already exists", label))
        } else {
            Ok(())
        }
    }

    pub fn label_address(&self, label: &str) -> Option<usize> {
        self.labels.get(label).copied()
    }

    pub fn resolve(&mut self) -> Result<&mut Self, String> {
        let resolved = self
            .instructions
            .iter()
            .enumerate()
            .map(|(i, maybe_instruction)| {
                maybe_instruction
                    .as_ref()
                    .map(|instruction| instruction.resolve(i, &self.labels))
                    .transpose()
            })
            .collect::<Result<_, String>>()?;

        self.resolved = Some(resolved);

        Ok(self)
    }
}

impl fmt::Display for Core {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self.resolved {
                None => &self.instructions,
                Some(instructions) => instructions,
            }
            .iter()
            .filter_map(Option::as_ref)
            .fold(String::new(), |result, instruction| {
                result + &instruction.to_string() + "\n"
            })
        )
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

    #[test]
    fn labels() {
        let mut core = Core::new(200);

        core.add_label(123, "baz".into()).expect("Should add baz");
        core.add_label(0, "foo".into()).expect("Should add foo");
        core.add_label(0, "bar".into()).expect("Should add bar");

        core.add_label(256, "goblin".into())
            .expect_err("Should fail to add labels > 200");
        core.add_label(5, "baz".into())
            .expect_err("Should error duplicate label");

        assert_eq!(core.label_address("foo").unwrap(), 0);
        assert_eq!(core.label_address("bar").unwrap(), 0);

        // The _last_ version of a label will be the one we use
        assert_eq!(core.label_address("baz").unwrap(), 5);

        assert!(core.label_address("goblin").is_none());
        assert!(core.label_address("never_mentioned").is_none());
    }

    #[test]
    fn resolve_failure() {
        let mut core = Core::new(10);

        core.add_label(0, "foo".into()).expect("Should add foo");

        core.set(
            5,
            Instruction {
                field_a: Field {
                    value: Value::Label("not_real".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        core.resolve().expect_err("Should fail to resolve");
    }

    #[test]
    fn resolve_labels() {
        let mut core = Core::new(10);

        core.add_label(0, "foo".into()).expect("Should add foo");
        core.add_label(7, "baz".into()).expect("Should add baz");

        core.set(
            3,
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

        core.resolve().expect("Should resolve all labels in core");

        assert_eq!(
            core.get(3).expect("Should have instruction at pos 5"),
            Instruction {
                field_a: Field {
                    value: Value::Literal(4),
                    ..Default::default()
                },
                field_b: Field {
                    value: Value::Literal(-3),
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    }
}
