//! Utilities for handling different types of address modes
//! Based on definitions in docs/icws94.txt:891
//!
//! All calls require a `&mut Core` because of the addressing modes which have
//! pre- and post-increment side effects.

use corewars_core::load_file::{AddressMode, Field, Instruction, Offset};

use super::Core;

/// Get the *relative* offset of the instruction pointed to by the A-field of the
/// given instruction. The core may be modified as a result of this lookup due
/// to the increment/decrement addressing modes.
///
/// As a result, calling code should only call this function once per step,
/// and get a clone/reference to the actual instruction as needed.
pub fn a_pointer(core: &mut Core, program_counter: Offset) -> Offset {
    let a_field = core.get_offset(program_counter).a_field.clone();
    resolve_pointer(core, program_counter, &a_field)
}

/// Get the *relative* offset of the instruction pointed to by the B-field of the
/// given instruction. The core may be modified as a result of this lookup due
/// to the increment/decrement addressing modes.
///
/// For this reason, calling code should only call this function once per step,
/// and get a clone/reference to the actual instruction as needed.
pub fn b_pointer(core: &mut Core, program_counter: Offset) -> Offset {
    let b_field = core.get_offset(program_counter).b_field.clone();
    resolve_pointer(core, program_counter, &b_field)
}

fn resolve_pointer(core: &mut Core, program_counter: Offset, field: &Field) -> Offset {
    use AddressMode::*;

    let address_mode = field.address_mode;

    match address_mode {
        Immediate => program_counter,
        Direct => program_counter + field.unwrap_value(),
        IndirectA | IndirectB => {
            let pointer_location = program_counter + field.unwrap_value();

            let pointer_value = if address_mode == IndirectA {
                core.get_offset(pointer_location).a_field.unwrap_value()
            } else {
                core.get_offset(pointer_location).b_field.unwrap_value()
            };

            pointer_location + pointer_value
        }
        PreDecIndirectA | PreDecIndirectB => {
            let pointer_location = program_counter + field.unwrap_value();

            let pointer_value = if address_mode == PreDecIndirectA {
                core.get_offset(pointer_location).a_field.unwrap_value()
            } else {
                core.get_offset(pointer_location).b_field.unwrap_value()
            };

            let decremented = core.offset(pointer_value) - 1;

            if address_mode == PreDecIndirectA {
                core.get_offset_mut(pointer_location)
                    .a_field
                    .set_value(decremented);
            } else {
                core.get_offset_mut(pointer_location)
                    .b_field
                    .set_value(decremented)
            };

            pointer_location + decremented
        }
        PostIncIndirectA | PostIncIndirectB => {
            let pointer_location = program_counter + field.unwrap_value();

            let pointer_value = if address_mode == PostIncIndirectA {
                core.get_offset(pointer_location).a_field.unwrap_value()
            } else {
                core.get_offset(pointer_location).b_field.unwrap_value()
            };

            let pre_increment = core.offset(pointer_value);

            if address_mode == PostIncIndirectA {
                core.get_offset_mut(pointer_location)
                    .a_field
                    .set_value(pre_increment + 1);
            } else {
                core.get_offset_mut(pointer_location)
                    .b_field
                    .set_value(pre_increment + 1)
            };

            pointer_location + pre_increment
        }
    }
}

#[cfg(test)]
mod tests {
    use corewars_core::load_file::Opcode;

    use test_case::test_case;

    use super::*;

    use super::super::tests::build_core;

    #[test]
    fn immediate_mode() {
        let mut core = build_core("dat #1, #2");
        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&mut core, pc), core.offset(0));
        assert_eq!(b_pointer(&mut core, pc), core.offset(0));
        assert_eq!(core.get(0), &instruction);
    }

    #[test]
    fn direct_mode() {
        let mut core = build_core("dat $1, $2");
        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&mut core, pc), core.offset(1));
        assert_eq!(b_pointer(&mut core, pc), core.offset(2));

        assert_eq!(core.get(0), &instruction);
    }

    #[test_case("*", 4, 7; "a")]
    #[test_case("@", 5, 8; "b")]
    fn indirect_mode(modifier: &str, expected_a: i32, expected_b: i32) {
        let mut core = build_core(&format!(
            "
            dat {0}1, {0}2
            dat #3, #4
            dat #5, #6
            ",
            modifier
        ));

        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(a_pointer(&mut core, pc), core.offset(expected_a));
        assert_eq!(b_pointer(&mut core, pc), core.offset(expected_b));

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

        assert_eq!(a_pointer(&mut core, pc), core.offset(3));
        assert_eq!(b_pointer(&mut core, pc), core.offset(6));

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

        assert_eq!(a_pointer(&mut core, pc), core.offset(4));
        assert_eq!(b_pointer(&mut core, pc), core.offset(7));

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

        assert_eq!(a_pointer(&mut core, pc), core.offset(4));
        assert_eq!(b_pointer(&mut core, pc), core.offset(7));

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

        assert_eq!(a_pointer(&mut core, pc), core.offset(5));
        assert_eq!(b_pointer(&mut core, pc), core.offset(8));

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
