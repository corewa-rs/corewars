use std::{fs, process::Command};

use assert_cmd::prelude::*;
use assert_fs::NamedTempFile;
use indoc::indoc;
use predicates::prelude::*;

static EXPECTED_OUT: &str = indoc!(
    "
    MOV.I $0, $1
    DAT.F $1, $2
    MOV.I $1, $0
    JMP.B $16, #0
    JMP.B $1, #0
    ADD.AB #1, @2
    SUB.F $3, $4
    MUL.F $5, $6
    DIV.F $7, $8
    MOD.F $9, $10
    JMZ.B $0, #0
    JMN.B $0, #0
    DJN.B $0, #0
    CMP.B $0, #0
    SEQ.B $0, #0
    SNE.B $0, #0
    SLT.B $0, #0
    SPL.B $0, #0
    NOP.B $0, #0
    MOV.A $1, $2
    MOV.B $1, $2
    MOV.AB $1, $2
    MOV.BA $1, $2
    MOV.F $1, $2
    MOV.X $1, $2
    MOV.I $1, $2
    JMP.B $-7, #0
    "
);

#[test]
fn help() {
    Command::cargo_bin(assert_cmd::crate_name!())
        .unwrap()
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Parse, assemble, and save Redcode files",
        ));
}

#[test]
fn help_dump() {
    Command::cargo_bin(assert_cmd::crate_name!())
        .unwrap()
        .arg("help")
        .arg("dump")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Save/print a program in 'load file' format",
        ));
}

#[test]
fn dump_stdout() {
    Command::cargo_bin(assert_cmd::crate_name!())
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/data/test.red")
        .arg("dump")
        .arg("--output-file")
        .arg("-")
        .assert()
        .success()
        .stdout(predicate::str::contains(EXPECTED_OUT));
}

#[test]
fn dump_file() {
    let out_file = NamedTempFile::new("out.red").expect("Failed to create tempfile");

    Command::cargo_bin(assert_cmd::crate_name!())
        .unwrap()
        .arg("tests/data/test.red")
        .arg("dump")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("--output-file")
        .arg(out_file.path())
        .assert()
        .success();

    let file_contents = fs::read_to_string(out_file.path()).expect("Failed to read output file");
    assert_eq!(file_contents, EXPECTED_OUT);
}
