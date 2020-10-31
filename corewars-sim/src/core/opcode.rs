//! Opcode-specific logic to run during a simulation step.

use std::cell::Cell;

use corewars_core::load_file::{Offset, Opcode};

use super::address;
use super::modifier;
use super::{Core, ExecutionError};

#[derive(Debug)]
pub struct Executed {
    pub program_counter_offset: Option<Offset>,
}

pub fn execute(core: &mut Core) -> Result<Executed, ExecutionError> {
    let instruction = core.current_instruction();
    let zero = core.offset(0);
    let program_counter_offset = Cell::new(None);

    let a_pointer = address::a_pointer(core, &instruction);
    let b_pointer = address::b_pointer(core, &instruction);

    let program_counter = core.program_counter;

    // See docs/icws94.txt:1113 for detailed description of each opcode
    match instruction.opcode {
        // Process control/miscellaneous opcodes
        Opcode::Dat => return Err(ExecutionError::ExecuteDat),
        Opcode::Mov => modifier::execute_on_instructions(
            core,
            a_pointer,
            b_pointer,
            |a, _b| Some(a),
            |a, _b| Some(a),
        ),
        Opcode::Nop => {}
        Opcode::Spl => unimplemented!(
            "This instruction will be complex to implement, as its behavior \
             does not really match other opcodes'"
        ),

        // Infallible arithmetic
        Opcode::Add => modifier::execute_on_fields(core, a_pointer, b_pointer, |a, b| Some(a + b)),
        Opcode::Mul => modifier::execute_on_fields(core, a_pointer, b_pointer, |a, b| Some(a * b)),
        Opcode::Sub => modifier::execute_on_fields(core, a_pointer, b_pointer, |a, b| Some(a - b)),

        // Fallible arithmetic
        Opcode::Div => {
            let mut div_result = Ok(());
            modifier::execute_on_fields(core, a_pointer, b_pointer, |a, b| {
                if b.value() == 0 {
                    div_result = Err(ExecutionError::DivideByZero);
                    None
                } else {
                    Some(a / b)
                }
            });
            div_result?;
        }
        Opcode::Mod => {
            let mut rem_result = Ok(());
            modifier::execute_on_fields(core, a_pointer, b_pointer, |a, b| {
                if b.value() == 0 {
                    rem_result = Err(ExecutionError::DivideByZero);
                    None
                } else {
                    Some(a % b)
                }
            });
            rem_result?;
        }

        // Skipping control flow opcodes
        Opcode::Cmp | Opcode::Seq => {
            program_counter_offset.set(core.offset(1).into());
            modifier::execute_on_instructions(
                core,
                a_pointer,
                b_pointer,
                |a, b| {
                    if a != b {
                        program_counter_offset.set(None)
                    }
                    None
                },
                // For e.g. F and I, all fields must match
                |a, b| {
                    if a != b {
                        program_counter_offset.set(None);
                    }
                    None
                },
            )
        }
        Opcode::Slt => {
            program_counter_offset.set(core.offset(1).into());
            modifier::execute_on_fields(core, a_pointer, b_pointer, |a, b| {
                if a.value() >= b.value() {
                    program_counter_offset.set(None);
                }
                None
            })
        }
        Opcode::Sne => {
            let next_instruction = Some(core.offset(1));
            modifier::execute_on_instructions(
                core,
                a_pointer,
                b_pointer,
                |a, b| {
                    if a != b {
                        program_counter_offset.set(next_instruction);
                    }
                    None
                },
                |a, b| {
                    if a != b {
                        program_counter_offset.set(next_instruction);
                    }
                    None
                },
            )
        }

        // Jumping control flow opcodes
        // These subtract the current program counter since this offset will be added to it later
        Opcode::Djn => modifier::execute_on_fields(core, a_pointer, b_pointer, |_a, b| {
            let decremented = b - 1i32;
            if decremented != zero {
                program_counter_offset.set((a_pointer - program_counter).into());
            }
            Some(decremented)
        }),
        Opcode::Jmn => modifier::execute_on_fields(core, a_pointer, b_pointer, |_a, b| {
            if b != zero {
                program_counter_offset.set((a_pointer - program_counter).into());
            }
            None
        }),
        Opcode::Jmp => {
            program_counter_offset.set((a_pointer - program_counter).into());
        }
        Opcode::Jmz => {
            modifier::execute_on_fields(core, a_pointer, b_pointer, |_a, b| {
                if b == zero {
                    program_counter_offset.set((a_pointer - program_counter).into());
                }
                None
            });
        }
    }

    Ok(Executed {
        program_counter_offset: program_counter_offset.get(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::tests::build_core;

    use corewars_core::load_file::{Field, Instruction, Modifier, Opcode};

    use test_case::test_case;

    mod process {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn execute_dat() {
            let mut core = build_core("dat #0, #0");
            let err = execute(&mut core).unwrap_err();
            assert_eq!(err, ExecutionError::ExecuteDat);
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

            let result = execute(&mut core).expect("Failed to execute");
            assert!(result.program_counter_offset.is_none());

            assert_eq!(
                &core.instructions[..4],
                &vec![
                    instruction.clone(),
                    instruction,
                    Default::default(),
                    Default::default(),
                ][..]
            );
        }

        #[test]
        fn execute_nop() {
            let mut core = build_core("nop #0, #0");
            let result = execute(&mut core).unwrap();
            assert!(result.program_counter_offset.is_none());
        }
    }

    #[test_case("add", 3; "add")]
    #[test_case("sub", 7999; "sub")]
    #[test_case("mul", 2; "mul")]
    fn execute_infallible_arithmetic(opcode: &str, expected_result: i32) {
        use pretty_assertions::assert_eq;

        let mut core = build_core(&format!(
            "
                {}.a $1, $2
                dat #1, #0
                dat #2, #0
                ",
            opcode
        ));

        let result = execute(&mut core).unwrap();
        assert!(result.program_counter_offset.is_none());

        assert_eq!(
            *core.get(2),
            Instruction::new(
                Opcode::Dat,
                Field::immediate(expected_result),
                Field::immediate(0)
            )
        );
    }

    mod fallible_arithmetic {
        use super::*;

        use super::test_case;

        use pretty_assertions::assert_eq;

        #[test]
        fn execute_div() {
            let mut core = build_core(
                "
                div $1, $2
                dat #8, #7
                dat #4, #2
                ",
            );

            let result = execute(&mut core).unwrap();
            assert!(result.program_counter_offset.is_none());

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
                div.f   $1, $2
                dat     #4, #6
                ",
            );

            core.set(2, divisor);

            let err = execute(&mut core).unwrap_err();

            assert_eq!(err, ExecutionError::DivideByZero);
            assert_eq!(core.get(2), &result)
        }

        #[test]
        fn execute_mod() {
            let mut core = build_core(
                "
                mod $1, $2
                dat #8, #7
                dat #4, #2
                ",
            );

            let result = execute(&mut core).unwrap();
            assert!(result.program_counter_offset.is_none());

            assert_eq!(
                *core.get(2),
                Instruction::new(Opcode::Dat, Field::immediate(0), Field::immediate(1)),
            )
        }

        #[test_case(
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(4)),
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(2))
            ; "a_zero"
        )]
        #[test_case(
            Instruction::new(Opcode::Dat, Field::direct(3), Field::direct(0)),
            Instruction::new(Opcode::Dat, Field::direct(1), Field::direct(0))
            ; "b_zero"
        )]
        #[test_case(
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0)),
            Instruction::new(Opcode::Dat, Field::direct(0), Field::direct(0))
            ; "both_zero"
        )]
        fn execute_mod_by_zero(divisor: Instruction, result: Instruction) {
            use pretty_assertions::assert_eq;

            let mut core = build_core(
                "
                mod.f   $1, $2
                dat     #4, #6
                ",
            );

            core.set(2, divisor);

            let err = execute(&mut core).unwrap_err();

            assert_eq!(err, ExecutionError::DivideByZero);
            assert_eq!(core.get(2), &result)
        }
    }

    mod skipping {
        use super::*;

        use super::test_case;

        #[test_case(
            "
            cmp.f   $1, $2
            dat     #0, #1
            dat     #0, #1
            ",
            Some(1)
            ; "cmp_equal"
        )]
        #[test_case(
            "
            seq.f   $1, $2
            dat     #0, #1
            dat     #0, #1
            ",
            Some(1)
            ; "seq_equal"
        )]
        #[test_case(
            "
            cmp.f   $1, $2
            dat     #0, #1
            dat     #1, #1
            ",
            None
            ; "cmp_unequal"
        )]
        #[test_case(
            "
            seq.f   $1, $2
            dat     #0, #1
            dat     #2, #0
            ",
            None
            ; "seq_unequal"
        )]
        #[test_case(
            "
            sne.f   $1, $2
            dat     #0, #1
            dat     #0, #1
            ",
            None
            ; "sne_equal"
        )]
        #[test_case(
            "
            sne.f   $1, $2
            dat     #0, #1
            dat     #1, #1
            ",
            Some(1)
            ; "sne_unequal"
        )]
        fn execute_skip_equality(program: &str, expected_offset: Option<i32>) {
            use pretty_assertions::assert_eq;

            let mut core = build_core(program);
            let expected_offset = expected_offset.map(|o| core.offset(o));
            let result = execute(&mut core).expect("Error executing opcode");

            assert_eq!(result.program_counter_offset, expected_offset);
        }

        #[test_case(
            "
            slt.a   $1, $2
            dat     #2, #0
            dat     #1, #0
            "
            ; "equal"
        )]
        #[test_case(
            "
            slt.a   $1, $2
            dat     #2, #0
            dat     #1, #0
            "
            ; "greater_than"
        )]
        fn execute_slt_no_skip(program: &str) {
            let mut core = build_core(program);
            let result = execute(&mut core).unwrap();
            assert!(result.program_counter_offset.is_none());
        }

        #[test]
        fn execute_slt_less_than() {
            let mut core = build_core(
                "
                slt.a   $1, $2
                dat     #1, #0
                dat     #2, #0
                ",
            );
            let result = execute(&mut core).unwrap();
            assert_eq!(result.program_counter_offset, Some(core.offset(1)));
        }
    }

    mod jumping {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn execute_djn_no_jump() {
            let mut core = build_core(
                "
                dat #0, #0
                djn.a $2, $1
                dat #1, #1
                nop #0, #0
                ",
            );
            core.program_counter = core.offset(1);

            let result = execute(&mut core).unwrap();

            assert_eq!(result.program_counter_offset, None);
            assert_eq!(
                &core.instructions[1..4],
                &vec![
                    Instruction {
                        opcode: Opcode::Djn,
                        modifier: Modifier::A,
                        a_field: Field::direct(2),
                        b_field: Field::direct(1)
                    },
                    Instruction::new(Opcode::Dat, Field::immediate(0), Field::immediate(1)),
                    Instruction::new(Opcode::Nop, Field::immediate(0), Field::immediate(0)),
                ][..]
            );
        }

        #[test]
        fn execute_djn_with_jump() {
            let mut core = build_core(
                "
                dat #0, #0
                djn.a $2, $1
                dat #3, #1
                nop #0, #0
                ",
            );
            core.program_counter = core.offset(1);

            let result = execute(&mut core).unwrap();

            assert_eq!(result.program_counter_offset, Some(core.offset(2)));
            assert_eq!(
                &core.instructions[1..4],
                &vec![
                    Instruction {
                        opcode: Opcode::Djn,
                        modifier: Modifier::A,
                        a_field: Field::direct(2),
                        b_field: Field::direct(1)
                    },
                    Instruction::new(Opcode::Dat, Field::immediate(2), Field::immediate(1)),
                    Instruction::new(Opcode::Nop, Field::immediate(0), Field::immediate(0)),
                ][..]
            );
        }

        #[test]
        fn execute_jmn_no_jump() {
            let mut core = build_core(
                "
                dat #0, #0
                jmn.a $2, $1
                dat #0, #1
                nop #0, #0
                ",
            );
            core.program_counter = core.offset(1);

            let result = execute(&mut core).unwrap();

            assert_eq!(result.program_counter_offset, None);
        }

        #[test]
        fn execute_jmn_with_jump() {
            let mut core = build_core(
                "
                dat #0, #0
                jmn.a $2, $1
                dat #1, #1
                nop #0, #0
                ",
            );
            core.program_counter = core.offset(1);

            let result = execute(&mut core).unwrap();

            assert_eq!(result.program_counter_offset, Some(core.offset(2)));
        }

        #[test]
        fn execute_jmp() {
            let mut core = build_core(
                "
                dat #0, #0
                jmp $3, #0
                ",
            );
            core.program_counter = core.offset(1);

            let result = execute(&mut core).expect("Failed to execute");

            assert_eq!(result.program_counter_offset, Some(core.offset(3)));
            assert_eq!(
                &core.instructions[1..5],
                &vec![
                    Instruction::new(Opcode::Jmp, Field::direct(3), Field::immediate(0)),
                    Default::default(),
                    Default::default(),
                    Default::default()
                ][..]
            );
        }

        #[test]
        fn execute_jmz_no_jump() {
            let mut core = build_core(
                "
                dat #0, #0
                jmz.a $2, $1
                dat #1, #1
                nop #0, #0
                ",
            );
            core.program_counter = core.offset(1);

            let result = execute(&mut core).unwrap();

            assert_eq!(result.program_counter_offset, None);
        }

        #[test]
        fn execute_jmz_with_jump() {
            let mut core = build_core(
                "
                dat #0, #0
                jmz.a $2, $1
                dat #0, #1
                nop #0, #0
                ",
            );
            core.program_counter = core.offset(1);

            let result = execute(&mut core).unwrap();

            assert_eq!(result.program_counter_offset, Some(core.offset(2)));
        }
    }
}
