use std::fs;
use std::path::Path;

use pretty_assertions::assert_eq;

use corewars_sim::Core;

#[test]
fn validate_redcode() {
    // Workaround for the fact that `test_resources` paths are based on workspace Cargo.toml
    // https://github.com/frehberg/test-generator/issues/6
    let input_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("testdata/input/simple/validate.redcode");

    let input = fs::read_to_string(input_file).unwrap();
    let warrior = corewars_parser::parse(&input).unwrap();

    // hmm, it would be useful to keep the labelmap around for analysis here...

    let mut core = Core::new(8_000).unwrap();
    core.load_warrior(&warrior).unwrap();

    eprintln!("Before run:\n{:?}\n==============================", core);

    // If the run fails, check the flag output to see where it failed
    if let Err(error) = core.run(10_000) {
        eprintln!(
            "Error '{}' after {} cycles have run:\n{:?}",
            error,
            core.steps_taken(),
            core
        );

        // "flag" label == 87 which should show why the test failed,
        // although it tends to be
        let flag = core.get(87);
        assert_eq!(
            (flag.a_field.unwrap_value(), flag.b_field.unwrap_value()),
            (0, 0)
        );
    }
}
