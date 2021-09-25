use std::fs;
use std::path::{Path, PathBuf};

use normalize_line_endings::normalized;
use pretty_assertions::assert_eq;
use test_generator::test_resources;

use corewars_core::load_file::PseudoOpcode;
use corewars_parser::Result as ParseResult;

#[test_resources("testdata/input/simple/*.redcode")]
#[test_resources("testdata/input/wilkie/*.redcode")]
#[test_resources("testdata/input/wilmoo/*.redcode")]
fn read_dir(input_file: &str) {
    // Workaround for the fact that `test_resources` paths are based on workspace Cargo.toml
    // https://github.com/frehberg/test-generator/issues/6
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(current_dir).unwrap();

    let input = fs::read_to_string(input_file)
        .unwrap_or_else(|err| panic!("Unable to read file {:?}: {:?}", input_file, err));

    let expected_out_file = PathBuf::from(input_file.replace("input", "expected_output"));

    let expected_output = fs::read_to_string(&expected_out_file).map_or_else(
        |err| panic!("Unable to read file {:?}: {:?}", input_file, err),
        |s| normalized(s.trim().chars()).collect::<String>(),
    );

    let parsed_warrior = match corewars_parser::parse(&input) {
        ParseResult::Ok(core, _) => core,
        ParseResult::Err(e, _) => panic!("Parse error:\n{}", e),
    };

    let mut core = corewars_sim::Core::default();
    core.load_warrior(&parsed_warrior)
        .expect("Failed to load warrior into core");

    let program_subset = &core[0..(parsed_warrior.len() as usize)];

    // This is kinda cheaty, but is the same impl as Program::fmt
    let org = format!(
        "{:<8}{}",
        PseudoOpcode::Org,
        parsed_warrior.program.origin.unwrap_or_default(),
    );

    let actual_lines: Vec<String> = std::iter::once(org)
        .chain(program_subset.iter().map(ToString::to_string))
        .collect();

    let expected_lines: Vec<&str> = expected_output.lines().collect();

    assert_eq!(expected_lines, actual_lines);
}
