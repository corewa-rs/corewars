//! Implementation details specific to opcode modifiers used during core simulation.

use corewars_core::load_file::{Instruction, Modifier, Offset};

use super::Core;

// These helper functions are based on definitions in docs/icws94.txt:891

fn a_instruction(core: &Core, instruction: &Instruction) -> Instruction {
    let a_pointer = instruction.a_field.unwrap_value();
    core.get_offset(core.program_counter + a_pointer).clone()
}

fn b_instruction(core: &Core, instruction: &Instruction) -> Instruction {
    let b_pointer = instruction.b_field.unwrap_value();
    core.get_offset(core.program_counter + b_pointer).clone()
}

fn b_target<'a>(core: &'a mut Core, instruction: &Instruction) -> &'a mut Instruction {
    let b_pointer = instruction.b_field.unwrap_value();
    core.get_offset_mut(core.program_counter + b_pointer)
}

/// Execute a given operation (`FieldOp`) on a given instruction. This is just a convenience
/// shortcut for `execute_on_instruction` without an `InstructionOp`
pub fn execute_on_fields<FieldOp>(core: &mut Core, field_op: FieldOp)
where
    FieldOp: FnMut(Offset, Offset) -> Option<Offset>,
{
    execute_on_instructions::<_, fn(_, _) -> _, _>(core, field_op, None)
}

/// Execute a given operation (`FieldOp`) on a given instruction. The `field_op`
/// and `instruction_op` arguments are expected to be closures taking an `a` and `b`
/// argument and returning the new value to set in the `b` instruction, if any.
pub fn execute_on_instructions<FieldOp, InstructionOp, OptionalInstructionOp>(
    core: &mut Core,
    mut field_op: FieldOp,
    instruction_op: OptionalInstructionOp,
) where
    FieldOp: FnMut(Offset, Offset) -> Option<Offset>,
    InstructionOp: FnMut(Instruction, Instruction) -> Option<Instruction>,
    OptionalInstructionOp: Into<Option<InstructionOp>>,
{
    let instruction = core.current_instruction();

    // TODO handle address modes during deref of these "pointer"s
    // For now it's always treated as direct addressing mode

    let a_value_a_offset = core.offset(a_instruction(core, &instruction).a_field.unwrap_value());
    let a_value_b_offset = core.offset(a_instruction(core, &instruction).b_field.unwrap_value());
    let b_value_a_offset = core.offset(b_instruction(core, &instruction).a_field.unwrap_value());
    let b_value_b_offset = core.offset(b_instruction(core, &instruction).b_field.unwrap_value());

    let a_instruction = a_instruction(core, &instruction);

    match instruction.modifier {
        Modifier::A => {
            if let Some(res) = field_op(a_value_a_offset, b_value_a_offset) {
                b_target(core, &instruction).a_field.set_value(res);
            }
        }
        Modifier::B => {
            if let Some(res) = field_op(a_value_b_offset, b_value_b_offset) {
                b_target(core, &instruction).b_field.set_value(res);
            }
        }
        Modifier::AB => {
            if let Some(res) = field_op(a_value_a_offset, b_value_b_offset) {
                b_target(core, &instruction).b_field.set_value(res);
            }
        }
        Modifier::BA => {
            if let Some(res) = field_op(a_value_b_offset, b_value_a_offset) {
                b_target(core, &instruction).a_field.set_value(res);
            }
        }
        Modifier::F | Modifier::I => {
            let b_instruction = b_instruction(core, &instruction);
            let b_target = b_target(core, &instruction);

            if let Some(a_res) = field_op(a_value_a_offset, b_value_a_offset) {
                b_target.a_field.set_value(a_res);
            }
            if let Some(b_res) = field_op(a_value_b_offset, b_value_b_offset) {
                b_target.b_field.set_value(b_res);
            }

            if instruction.modifier == Modifier::I {
                if let Some(mut instruction_op) = instruction_op.into() {
                    if let Some(res) = instruction_op(a_instruction, b_instruction) {
                        b_target.opcode = res.opcode;
                        b_target.modifier = res.modifier;
                    }
                }
            }
        }
        Modifier::X => {
            let b_target = b_target(core, &instruction);

            if let Some(a_res) = field_op(a_value_b_offset, b_value_a_offset) {
                b_target.a_field.set_value(a_res);
            }
            if let Some(b_res) = field_op(a_value_a_offset, b_value_b_offset) {
                b_target.b_field.set_value(b_res);
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
        execute_on_fields(&mut core, |a, b| {
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
        execute_on_instructions(
            &mut core,
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
