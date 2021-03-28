//! Utilities for handling different types of address modes
//! Based on definitions in docs/icws94.txt:891
//!
//! All calls require a `&mut Core` because of the addressing modes which have
//! pre- and post-increment side effects.

use corewars_core::load_file::{AddressMode, Field, Offset};

use super::Core;

/// Get the *relative* offset of the instruction pointed to by the A-field of the
/// given instruction.
pub fn resolve_a_pointer(core: &Core, program_counter: Offset) -> Offset {
    let a_field = core.get_offset(program_counter).a_field.clone();
    resolve_pointer(core, program_counter, &a_field)
}

/// Get the *relative* offset of the instruction pointed to by the B-field of the
/// given instruction.
pub fn resolve_b_pointer(core: &Core, program_counter: Offset) -> Offset {
    let b_field = core.get_offset(program_counter).b_field.clone();
    resolve_pointer(core, program_counter, &b_field)
}

fn resolve_pointer(core: &Core, program_counter: Offset, field: &Field) -> Offset {
    use AddressMode::*;

    let address_mode = field.address_mode;
    let field_value = field.unwrap_value();
    let pointed_to = core.get_offset(program_counter + field_value);

    let offset = match address_mode {
        Immediate => 0,
        Direct => field_value,
        IndirectA | PostIncIndirectA => field_value + pointed_to.a_field.unwrap_value(),
        IndirectB | PostIncIndirectB => field_value + pointed_to.b_field.unwrap_value(),
        PreDecIndirectA => field_value + pointed_to.a_field.unwrap_value() - 1,
        PreDecIndirectB => field_value + pointed_to.b_field.unwrap_value() - 1,
    };

    program_counter + offset
}

/// Whether an address mode is being applied before or after evaluation
pub enum EvalTime {
    Pre,
    Post,
}

pub fn apply_a_pointer(core: &mut Core, program_counter: Offset, eval_time: EvalTime) {
    let a_field = core.get_offset(program_counter).a_field.clone();
    apply_pointer(core, program_counter, &a_field, eval_time);
}

pub fn apply_b_pointer(core: &mut Core, program_counter: Offset, eval_time: EvalTime) {
    let b_field = core.get_offset(program_counter).b_field.clone();
    apply_pointer(core, program_counter, &b_field, eval_time);
}

fn apply_pointer(core: &mut Core, program_counter: Offset, field: &Field, eval_time: EvalTime) {
    use AddressMode::*;

    let address_mode = field.address_mode;
    let pointer_location = program_counter + field.unwrap_value();

    let pointed_to = core.get_offset(pointer_location);
    let a_value = core.offset(pointed_to.a_field.unwrap_value());
    let b_value = core.offset(pointed_to.b_field.unwrap_value());

    let pointed_to = core.get_offset_mut(pointer_location);

    match (eval_time, address_mode) {
        (EvalTime::Pre, PreDecIndirectA) => pointed_to.a_field.set_value(a_value - 1),
        (EvalTime::Pre, PreDecIndirectB) => pointed_to.b_field.set_value(b_value - 1),
        (EvalTime::Post, PostIncIndirectA) => pointed_to.a_field.set_value(a_value + 1),
        (EvalTime::Post, PostIncIndirectB) => pointed_to.b_field.set_value(b_value + 1),
        _ => {}
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

        assert_eq!(resolve_a_pointer(&core, pc), core.offset(0));
        assert_eq!(resolve_b_pointer(&core, pc), core.offset(0));
        assert_eq!(core.get(0), &instruction);
    }

    #[test]
    fn direct_mode() {
        let core = build_core("dat $1, $2");
        let pc = core.offset(0);
        let instruction = core.get_offset(pc).clone();

        assert_eq!(resolve_a_pointer(&core, pc), core.offset(1));
        assert_eq!(resolve_b_pointer(&core, pc), core.offset(2));

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

        assert_eq!(resolve_a_pointer(&core, pc), core.offset(expected_a));
        assert_eq!(resolve_b_pointer(&core, pc), core.offset(expected_b));

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

        assert_eq!(resolve_a_pointer(&core, pc), core.offset(3));
        assert_eq!(resolve_b_pointer(&core, pc), core.offset(6));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );

        apply_a_pointer(&mut core, pc, EvalTime::Pre);
        apply_b_pointer(&mut core, pc, EvalTime::Pre);

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

        assert_eq!(resolve_a_pointer(&core, pc), core.offset(4));
        assert_eq!(resolve_b_pointer(&core, pc), core.offset(7));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );

        apply_a_pointer(&mut core, pc, EvalTime::Pre);
        apply_b_pointer(&mut core, pc, EvalTime::Pre);

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

        assert_eq!(resolve_a_pointer(&core, pc), core.offset(4));
        assert_eq!(resolve_b_pointer(&core, pc), core.offset(7));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );

        apply_a_pointer(&mut core, pc, EvalTime::Post);
        apply_b_pointer(&mut core, pc, EvalTime::Post);

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

        assert_eq!(resolve_a_pointer(&core, pc), core.offset(5));
        assert_eq!(resolve_b_pointer(&core, pc), core.offset(8));

        assert_eq!(core.get(0), &instruction);
        assert_eq!(
            core.get(1),
            &Instruction::new(Opcode::Dat, Field::immediate(3), Field::immediate(4))
        );
        assert_eq!(
            core.get(2),
            &Instruction::new(Opcode::Dat, Field::immediate(5), Field::immediate(6))
        );

        apply_a_pointer(&mut core, pc, EvalTime::Post);
        apply_b_pointer(&mut core, pc, EvalTime::Post);

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
