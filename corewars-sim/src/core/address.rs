//! Utilities for handling different types of address modes
//! Based on definitions in docs/icws94.txt:891
//!
//! All calls require a `&mut Core` because of the addressing modes which have
//! pre- and post-increment side effects.

use corewars_core::load_file::{AddressMode, Field, Offset};

use super::Core;

/// Get the *relative* offset of the instruction pointed to by the A-field of the
/// given instruction.
pub fn a_pointer(core: &Core, program_counter: Offset) -> Offset {
    let a_field = core.get_offset(program_counter).a_field.clone();
    resolve_pointer(core, program_counter, &a_field)
}

/// Get the *relative* offset of the instruction pointed to by the B-field of the
/// given instruction.
pub fn b_pointer(core: &Core, program_counter: Offset) -> Offset {
    let b_field = core.get_offset(program_counter).b_field.clone();
    resolve_pointer(core, program_counter, &b_field)
}

fn resolve_pointer(core: &Core, program_counter: Offset, field: &Field) -> Offset {
    use AddressMode::*;

    let address_mode = field.address_mode;

    let field_value = field.unwrap_value();

    let offset = match address_mode {
        Immediate => 0,
        Direct => field_value,
        IndirectA | IndirectB | PostIncIndirectA | PostIncIndirectB => {
            let pointed_to = core.get_offset(program_counter + field_value);

            let indirect_offset = match address_mode {
                IndirectA | PostIncIndirectA => pointed_to.a_field.unwrap_value(),
                IndirectB | PostIncIndirectB => pointed_to.b_field.unwrap_value(),
                _ => unreachable!(),
            };

            field_value + indirect_offset
        }
        PreDecIndirectA | PreDecIndirectB => {
            let pointed_to = core.get_offset(program_counter + field_value);

            let indirect_offset = match address_mode {
                PreDecIndirectA => pointed_to.a_field.unwrap_value(),
                PreDecIndirectB => pointed_to.b_field.unwrap_value(),
                _ => unreachable!(),
            };

            field_value + indirect_offset - 1
        }
    };

    program_counter + offset
}

/// Apply any changes due to postincrement/predecrement address modes on the A pointer.
pub fn apply_a_pointer(core: &mut Core, program_counter: Offset) {
    let a_field = core.get_offset(program_counter).a_field.clone();
    apply_pointer(core, program_counter, &a_field);
}

/// Apply any changes due to postincrement/predecrement address modes on the B pointer.
pub fn apply_b_pointer(core: &mut Core, program_counter: Offset) {
    let b_field = core.get_offset(program_counter).b_field.clone();
    apply_pointer(core, program_counter, &b_field);
}

fn apply_pointer(core: &mut Core, program_counter: Offset, field: &Field) {
    use AddressMode::*;

    let address_mode = field.address_mode;
    let pointer_location = program_counter + field.unwrap_value();

    if let PreDecIndirectA | PostIncIndirectA | PreDecIndirectB | PostIncIndirectB = address_mode {
        let zero = core.offset(0);

        let pointed_to = core.get_offset_mut(pointer_location);

        let new_value = zero
            + match address_mode {
                PreDecIndirectA => pointed_to.a_field.unwrap_value() - 1,
                PreDecIndirectB => pointed_to.b_field.unwrap_value() - 1,
                PostIncIndirectA => pointed_to.a_field.unwrap_value() + 1,
                PostIncIndirectB => pointed_to.b_field.unwrap_value() + 1,
                _ => unreachable!(),
            };

        match address_mode {
            PreDecIndirectA | PostIncIndirectA => {
                pointed_to.a_field.set_value(new_value);
            }
            PreDecIndirectB | PostIncIndirectB => {
                pointed_to.b_field.set_value(new_value);
            }
            _ => unreachable!(),
        };
    }
}

#[cfg(test)]
mod tests {
    use corewars_core::load_file::{Instruction, Opcode};

    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use super::*;

    use super::super::tests::build_core;

    #[test]
    fn immediate_mode() {
        let core = build_core("dat #1, #2");
        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&core, pc), core.offset(0));
        assert_eq!(b_pointer(&core, pc), core.offset(0));
        assert_eq!(core.get(0), &instruction);
    }

    #[test]
    fn direct_mode() {
        let core = build_core("dat $1, $2");
        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&core, pc), core.offset(1));
        assert_eq!(b_pointer(&core, pc), core.offset(2));

        assert_eq!(core.get(0), &instruction);
    }

    #[test_case("*", 4, 7; "a")]
    #[test_case("@", 5, 8; "b")]
    fn indirect_mode(modifier: &str, expected_a: i32, expected_b: i32) {
        use pretty_assertions::assert_eq;

        let core = build_core(&format!(
            "
            dat {0}1, {0}2
            dat #3, #4
            dat #5, #6
            ",
            modifier
        ));

        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&core, pc), core.offset(expected_a));
        assert_eq!(b_pointer(&core, pc), core.offset(expected_b));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );
    }

    #[test]
    fn indirect_predecrement_a() {
        let mut core = build_core(
            "
            dat {1, {2
            dat #3, #4
            dat #5, #6
            ",
        );

        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&core, pc), core.offset(3));
        assert_eq!(b_pointer(&core, pc), core.offset(6));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );

        apply_a_pointer(&mut core, pc);
        apply_b_pointer(&mut core, pc);

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(2), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(4), Field::immediate(6))
        );
    }

    #[test]
    fn indirect_predecrement_b() {
        let mut core = build_core(
            "
            dat <1, <2
            dat #3, #4
            dat #5, #6
            ",
        );

        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&core, pc), core.offset(4));
        assert_eq!(b_pointer(&core, pc), core.offset(7));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );

        apply_a_pointer(&mut core, pc);
        apply_b_pointer(&mut core, pc);

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(3))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(5))
        );
    }

    #[test]
    fn indirect_postincrement_a() {
        let mut core = build_core(
            "
            dat }1, }2
            dat #3, #4
            dat #5, #6
            ",
        );

        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&core, pc), core.offset(4));
        assert_eq!(b_pointer(&core, pc), core.offset(7));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );

        apply_a_pointer(&mut core, pc);
        apply_b_pointer(&mut core, pc);

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(4), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(6), Field::immediate(6))
        );
    }

    #[test]
    fn indirect_postincrement_b() {
        let mut core = build_core(
            "
            dat >1, >2
            dat #3, #4
            dat #5, #6
            ",
        );

        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&core, pc), core.offset(5));
        assert_eq!(b_pointer(&core, pc), core.offset(8));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );

        apply_a_pointer(&mut core, pc);
        apply_b_pointer(&mut core, pc);

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(5))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(7))
        );
    }
}
