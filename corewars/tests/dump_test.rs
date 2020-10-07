use std::fs;
use std::path::{Path, PathBuf};

use assert_that::assert_that;
use test_generator::test_resources;

use corewars::parser::Result as ParseResult;

#[test_resources("corewars/tests/data/input/simple/*.red")]
#[test_resources("corewars/tests/data/input/wilkie/*.red")]
#[test_resources("corewars/tests/data/input/wilmoo/*.red")]
fn read_dir(input_file: &str) {
    // Workaround for the fact that `test_resources` paths are based on workspace Cargo.toml
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(current_dir).unwrap();

    let input = fs::read_to_string(input_file)
        .unwrap_or_else(|err| panic!("Unable to read file {:?}: {:?}", input_file, err));

    let expected_out_file = PathBuf::from(input_file.replace("input", "expected_output"));
    if !expected_out_file.exists() {
        // TODO after #39 this shouldn't be needed
        eprintln!("No output file, skipping test");
        return;
    }

    let expected_output = fs::read_to_string(&expected_out_file)
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|err| panic!("Unable to read file {:?}: {:?}", input_file, err));

    let parsed_core = match corewars::parser::parse(&input) {
        ParseResult::Ok(core, _) => core,
        ParseResult::Err(e, _) => panic!("Parse error:\n{}", e),
    };

    assert_that!(
        &parsed_core.to_string().trim(),
        str::similar(expected_output)
    );
}
