//! Utilities for handling different types of address modes
//! Based on definitions in docs/icws94.txt:891
//!
//! All calls require a `&mut Core` because of the addressing modes which have
//! pre- and post-increment side effects.

use corewars_core::load_file::{AddressMode, Field, Instruction, Offset};

use super::Core;

pub fn a_instruction(core: &mut Core, instruction: &Instruction) -> Instruction {
    let a_pointer = resolve_pointer(core, &instruction.a_field);
    core.get_offset(a_pointer).clone()
}

pub fn b_instruction(core: &mut Core, instruction: &Instruction) -> Instruction {
    let b_pointer = resolve_pointer(core, &instruction.b_field);
    core.get_offset(b_pointer).clone()
}

pub fn b_target<'a>(core: &'a mut Core, instruction: &Instruction) -> &'a mut Instruction {
    let b_pointer = resolve_pointer(core, &instruction.b_field);
    core.get_offset_mut(b_pointer)
}

fn resolve_pointer(core: &mut Core, field: &Field) -> Offset {
    use AddressMode::*;
    let address_mode = field.address_mode;
    match address_mode {
        Immediate => core.program_counter,
        Direct => core.program_counter + field.unwrap_value(),
        IndirectA | IndirectB => {
            let pointer_location = core.program_counter + field.unwrap_value();

            let pointer_value = if address_mode == IndirectA {
                core.get_offset(pointer_location).a_field.unwrap_value()
            } else {
                core.get_offset(pointer_location).b_field.unwrap_value()
            };

            pointer_location + pointer_value
        }
        PreDecIndirectA | PreDecIndirectB => {
            let pointer_location = core.program_counter + field.unwrap_value();

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
            let pointer_location = core.program_counter + field.unwrap_value();

            let pointer_value = if address_mode == PreDecIndirectA {
                core.get_offset(pointer_location).a_field.unwrap_value()
            } else {
                core.get_offset(pointer_location).b_field.unwrap_value()
            };

            let pre_increment = core.offset(pointer_value);

            if address_mode == PreDecIndirectA {
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
