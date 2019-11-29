use std::{error::Error, fs, path::PathBuf};

use structopt::StructOpt;

use crate::parser;

#[derive(Debug, StructOpt)]
/// Parse and save Redcode files
struct CliOptions {
    /// Input file
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,

    /// Output file; defaults to stdout
    #[structopt(long, short, parse(from_os_str))]
    output_file: Option<PathBuf>,

    /// Whether or not labels, expressions, etc. should be resolved in the output
    #[structopt(long, short)]
    resolve: bool,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli_options = CliOptions::from_args();

    let input_program = fs::read_to_string(cli_options.input_file)?;

    let mut parsed_core = parser::parse(input_program.as_str())?;

    if cli_options.resolve {
        parsed_core.resolve()?;
    }

    if let Some(output_path) = cli_options.output_file {
        fs::write(output_path, format!("{}", parsed_core))?;
    } else {
        println!("{}", parsed_core);
    };

    Ok(())
}
