use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use lazy_static::lazy_static;
use normalize_line_endings::normalized;
use predicates::prelude::*;
use pretty_assertions::assert_eq;

lazy_static! {
    static ref EXPECTED_OUT: String = normalized(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/cli_out.redcode"
        ))
        .chars()
    )
    .collect();
}

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
            r#"Save/print a program in "load file" format"#,
        ));
}

#[test]
fn dump_stdout() {
    let cmd = Command::cargo_bin(assert_cmd::crate_name!())
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("../testdata/input/simple/basic.redcode")
        .arg("dump")
        .arg("--output-file")
        .arg("-")
        .assert()
        .success();

    let out_text = cmd.get_output().stdout.clone();

    let file_contents: String = normalized(String::from_utf8(out_text).unwrap().chars()).collect();
    assert_eq!(file_contents, &**EXPECTED_OUT);
}

#[test]
fn dump_file() {
    let out_file = assert_fs::NamedTempFile::new("out.redcode").expect("Failed to create tempfile");

    Command::cargo_bin(assert_cmd::crate_name!())
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("../testdata/input/simple/basic.redcode")
        .arg("dump")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("--output-file")
        .arg(out_file.path())
        .assert()
        .success();

    let file_contents: String =
        normalized(fs::read_to_string(out_file.path()).unwrap().chars()).collect();

    assert_eq!(file_contents, &**EXPECTED_OUT);
}
