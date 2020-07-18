extern crate assert_cmd;
extern crate assert_that;
extern crate predicates;

extern crate corewa_rs;

use std::process::Command;

use assert_cmd::prelude::*;
use assert_that::assert_that;
use predicates::prelude::*;

static EXPECTED_OUT: &str = include_str!("data/test_loadfile.red");

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
    Command::cargo_bin(assert_cmd::crate_name!())
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/data/test.red")
        .arg("dump")
        .arg("--output-file")
        .arg("-")
        .assert()
        .success()
        .stdout(predicate::str::similar(EXPECTED_OUT));
}

#[test]
fn dump_file() {
    let out_file = assert_fs::NamedTempFile::new("out.red").expect("Failed to create tempfile");

    Command::cargo_bin(assert_cmd::crate_name!())
        .unwrap()
        .arg("tests/data/test.red")
        .arg("dump")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("--output-file")
        .arg(out_file.path())
        .assert()
        .success();

    assert_that!(
        out_file.path(),
        str::similar(EXPECTED_OUT).from_utf8().from_file_path(),
    );
}
