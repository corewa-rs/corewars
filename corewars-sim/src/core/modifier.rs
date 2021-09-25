//! Implementation details specific to opcode modifiers used during core simulation.

use corewars_core::load_file::{Instruction, Modifier, Offset};

use super::address;
use super::Core;

/// A helper struct to execute an instruction using the proper modifiers.
/// This struct maintains the "registers" used for evaluating instructions
pub(super) struct Executor<'a> {
    core: &'a mut Core,
    program_counter: Offset,
    a_value: Instruction,
    b_value: Instruction,
    a_ptr: Offset,
    b_ptr: Offset,
}

impl<'a> Executor<'a> {
    /// Build a new executor for the given program offset of the given [`Core`].
    pub fn new(core: &'a mut Core, program_counter: Offset) -> Self {
        let a_ptr = address::resolve_a_pointer(core, program_counter);

        // NOTE: the order of evaluation is significant here: we create the "register"
        // by cloning the A operand before evaluating the B pointer, and all further
        // operations must use the buffered A operand, in case the B pointer evaluation
        // modifies memory
        address::apply_a_pointer(core, program_counter, address::EvalTime::Pre);
        let a_value = core.get_offset(a_ptr).clone();
        address::apply_a_pointer(core, program_counter, address::EvalTime::Post);

        let b_ptr = address::resolve_b_pointer(core, program_counter);

        address::apply_b_pointer(core, program_counter, address::EvalTime::Pre);
        let b_value = core.get_offset(b_ptr).clone();
        address::apply_b_pointer(core, program_counter, address::EvalTime::Post);

        Self {
            core,
            program_counter,
            a_value,
            b_value,
            a_ptr,
            b_ptr,
        }
    }

    /// Getter for the resolved A pointer
    pub fn a_ptr(&self) -> Offset {
        self.a_ptr
    }

    /// Execute a given operation (`FieldOp`) on a given instruction. This is a convenience
    /// shortcut for [`run_on_instructions`](Self::run_on_instructions) without an `InstructionOp`.
    pub fn run_on_fields<FieldOp>(self, field_op: FieldOp)
    where
        FieldOp: FnMut(Offset, Offset) -> Option<Offset>,
    {
        self.run_on_instructions::<_, fn(_, _) -> _, _>(field_op, None);
    }

    /// Execute a given operation (`FieldOp`) on a given instruction.
    /// `field_op` and `instruction_op` are closures taking an `a` and `b`
    /// argument and returning the new value to set in the `b` instruction, if any.
    pub fn run_on_instructions<FieldOp, InstructionOp, OptionalInstructionOp>(
        self,
        mut field_op: FieldOp,
        instruction_op: OptionalInstructionOp,
    ) where
        FieldOp: FnMut(Offset, Offset) -> Option<Offset>,
        InstructionOp: FnMut(Instruction, Instruction) -> Option<Instruction>,
        OptionalInstructionOp: Into<Option<InstructionOp>>,
    {
        let instruction = self.core.get_offset(self.program_counter).clone();

        let a_value_a_offset = self.core.offset(self.a_value.a_field.unwrap_value());
        let a_value_b_offset = self.core.offset(self.a_value.b_field.unwrap_value());

        let b_value_a_offset = self.core.offset(self.b_value.a_field.unwrap_value());
        let b_value_b_offset = self.core.offset(self.b_value.b_field.unwrap_value());

        let b_target = self.core.get_offset_mut(self.b_ptr);

        match instruction.modifier {
            Modifier::A => {
                if let Some(res) = field_op(a_value_a_offset, b_value_a_offset) {
                    b_target.a_field.set_value(res);
                }
            }
            Modifier::B => {
                if let Some(res) = field_op(a_value_b_offset, b_value_b_offset) {
                    b_target.b_field.set_value(res);
                }
            }
            Modifier::AB => {
                if let Some(res) = field_op(a_value_a_offset, b_value_b_offset) {
                    b_target.b_field.set_value(res);
                }
            }
            Modifier::BA => {
                if let Some(res) = field_op(a_value_b_offset, b_value_a_offset) {
                    b_target.a_field.set_value(res);
                }
            }
            Modifier::F | Modifier::I => {
                if let Some(a_res) = field_op(a_value_a_offset, b_value_a_offset) {
                    b_target.a_field.set_value(a_res);
                }
                if let Some(b_res) = field_op(a_value_b_offset, b_value_b_offset) {
                    b_target.b_field.set_value(b_res);
                }

                if instruction.modifier == Modifier::I {
                    if let Some(mut instruction_op) = instruction_op.into() {
                        if let Some(res) = instruction_op(self.a_value, b_target.clone()) {
                            b_target.opcode = res.opcode;
                            b_target.modifier = res.modifier;
                        }
                    }
                }
            }
            Modifier::X => {
                if let Some(a_res) = field_op(a_value_b_offset, b_value_a_offset) {
                    b_target.a_field.set_value(a_res);
                }
                if let Some(b_res) = field_op(a_value_a_offset, b_value_b_offset) {
                    b_target.b_field.set_value(b_res);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::tests::build_core;

    use corewars_core::load_file::{Field, Instruction, Opcode};

    use pretty_assertions::assert_eq;
    use test_case::test_case;

    #[test_case("a", 35, 6; "a")]
    #[test_case("b", 5, 46; "b")]
    #[test_case("ab", 5, 36; "ab")]
    #[test_case("ba", 45, 6; "ba")]
    #[test_case("f", 35, 46; "f")]
    #[test_case("x", 45, 36; "x")]
    fn field_modifier(modifier: &str, expected_a: i32, expected_b: i32) {
        use pretty_assertions::assert_eq;

        let mut core = build_core(&format!(
            "
            dat.{}  $1, $2
            dat     $3, $4
            sub.x   $5, $6
            ",
            modifier
        ));

        let zero = core.offset(0);
        let exec = Executor::new(&mut core, zero);

        exec.run_on_fields(|a, b| {
            // kinda hacky way to verify exact outputs but I guess it works...
            let string_ans = a.value().to_string() + &b.value().to_string();
            Some(zero + string_ans.parse::<i32>().unwrap())
        });

        assert_eq!(
            core.get(2),
            &Instruction {
                opcode: Opcode::Sub,
                modifier: Modifier::X,
                a_field: Field::direct(expected_a),
                b_field: Field::direct(expected_b),
            }
        );
    }

    #[test]
    fn instruction_modifier() {
        let mut core = build_core(
            "
            dat.i $1, $2
            add.f $3, $4
            sub.x $5, $6
            ",
        );

        let output = core.offset(0);
        let zero = core.offset(0);

        let exec = Executor::new(&mut core, zero);

        exec.run_on_instructions(
            |a, b| {
                let string_ans = a.value().to_string() + &b.value().to_string();
                Some(output + string_ans.parse::<i32>().unwrap())
            },
            |a: Instruction, b: Instruction| {
                assert_eq!(a.opcode, Opcode::Add);
                assert_eq!(b.opcode, Opcode::Sub);

                Some(Instruction {
                    opcode: Opcode::Nop,
                    modifier: Modifier::AB,
                    a_field: Field::direct(0),
                    b_field: Field::direct(0),
                })
            },
        );

        assert_eq!(
            core.get(2),
            &Instruction {
                opcode: Opcode::Nop,
                modifier: Modifier::AB,
                a_field: Field::direct(35),
                b_field: Field::direct(46),
            }
        );
    }
}
