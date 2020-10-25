//! Opcode-specific logic to run during a simulation step.

use std::cell::Cell;

use corewars_core::load_file::{Offset, Opcode};

use super::{modifier, Core, ExecutionError};

#[derive(Debug)]
pub struct Executed {
    pub program_counter_offset: Offset,
}

pub fn execute(core: &mut Core) -> Result<Executed, ExecutionError> {
    let instruction = core.current_instruction();
    let zero = core.offset(0);
    let program_counter_offset = Cell::new(zero);

    // See docs/icws94.txt:1113 for detailed description of each opcode
    match instruction.opcode {
        Opcode::Add => modifier::execute_on_fields(core, |a, b| Some(a + b)),
        Opcode::Cmp | Opcode::Seq => {
            program_counter_offset.set(core.offset(1));
            // For e.g. F and I, all fields must match
            modifier::execute_on_instructions(
                core,
                |a, b| {
                    if a != b {
                        program_counter_offset.set(zero)
                    }
                    None
                },
                |a, b| {
                    if a != b {
                        program_counter_offset.set(zero);
                    }
                    None
                },
            )
        }
        Opcode::Dat => return Err(ExecutionError::ExecuteDat),
        Opcode::Div => {
            let mut div_result = Ok(());
            modifier::execute_on_fields(core, |a, b| {
                if b.value() == 0 {
                    div_result = Err(ExecutionError::DivideByZero);
                    None
                } else {
                    Some(a / b)
                }
            });
            div_result?;
        }
        Opcode::Djn => todo!(),
        Opcode::Jmn => todo!(),
        Opcode::Jmp => {
            // Subtract one since we always increment by one anyway
            core.program_counter += instruction.a_field.unwrap_value() - 1;
        }

        Opcode::Jmz => todo!(),
        Opcode::Mod => todo!(),
        Opcode::Mov => modifier::execute_on_instructions(core, |a, _b| Some(a), |a, _b| Some(a)),
        Opcode::Mul => todo!(),
        Opcode::Nop => todo!(),
        Opcode::Slt => todo!(),
        Opcode::Sne => todo!(),
        Opcode::Spl => todo!(),
        Opcode::Sub => todo!(),
    }

    Ok(Executed {
        program_counter_offset: program_counter_offset.get(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::tests::build_core;

    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use corewars_core::load_file::{Field, Instruction, Modifier, Opcode};

    #[test]
    fn execute_dat() {
        let mut core = Core::new(4).unwrap();
        // Default instruction is DAT, so expect a termination error immediately
        assert_eq!(core.step().unwrap_err(), ExecutionError::ExecuteDat);
    }

    #[test]
    fn execute_div() {
        let mut core = build_core(
            "
            div $1, $2
            dat #8, #7
            dat #4, #2
            ",
        );

        core.step().unwrap();
        assert_eq!(
            *core.get(2),
            Instruction::new(Opcode::Dat, Field::immediate(2), Field::immediate(3)),
        )
    }

    #[test_case(
        Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(2)),
        Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(3))
        ; "a_zero"
    )]
    #[test_case(
        Instruction::new(Opcode::Dat, Field::direct(2), Field::direct(0)),
        Instruction::new(Opcode::Dat, Field::direct(2), Field::direct(0))
        ; "b_zero"
    )]
    #[test_case(
        Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0)),
        Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0))
        ; "both_zero"
    )]
    fn execute_div_by_zero(divisor: Instruction, result: Instruction) {
        use pretty_assertions::assert_eq;

        let mut core = build_core(
            "
            div.f   #1, #2
            dat     #4, #6
            ",
        );

        core.set(2, divisor);

        assert_eq!(core.step().unwrap_err(), ExecutionError::DivideByZero);
        assert_eq!(core.get(2), &result)
    }

    #[test]
    fn execute_mov() {
        let instruction = Instruction {
            opcode: Opcode::Mov,
            modifier: Modifier::I,
            a_field: Field::direct(0),
            b_field: Field::direct(1),
        };
        let mut core = build_core("mov.i $0, $1");

        assert_eq!(core.current_instruction(), instruction);

        core.step().unwrap();
        assert_eq!(core.current_instruction(), instruction);

        core.step().unwrap();
        assert_eq!(core.current_instruction(), instruction);
    }

    #[test]
    fn execute_jmp() {
        let mut core = build_core("jmp $3, #0");

        core.step().unwrap();

        assert_eq!(core.program_counter.value(), 3);
        assert_eq!(
            &core.instructions[..4],
            &vec![
                Instruction::new(Opcode::Jmp, Field::direct(3), Field::immediate(0)),
                Default::default(),
                Default::default(),
                Default::default()
            ][..]
        );
    }

    #[test_case(
        "
        cmp.f $1, $2
        dat #0, #1
        dat #0, #1
        ",
        2
        ; "cmp_equal"
    )]
    #[test_case(
        "
        seq.f $1, $2
        dat #0, #1
        dat #0, #1
        ",
        2
        ; "seq_equal"
    )]
    #[test_case(
        "
        cmp.f $1, $2
        dat #0, #1
        dat #1, #1
        ",
        1
        ; "cmp_unequal"
    )]
    #[test_case(
        "
        seq.f $1, $2
        dat #0, #1
        dat #2, #0
        ",
        1
        ; "seq_unequal"
    )]
    fn execute_seq(program: &str, expected_program_counter: u32) {
        let mut core = build_core(program);

        core.step().unwrap();

        pretty_assertions::assert_eq!(core.program_counter.value(), expected_program_counter);
    }
}
