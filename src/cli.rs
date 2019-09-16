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
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli_options = CliOptions::from_args();

    let input_program = fs::read_to_string(cli_options.input_file)?;

    let parsed_input = parser::parse(input_program.as_str())?;
    let parse_output = parsed_input.dump();

    match cli_options.output_file {
        Some(output_path) => fs::write(output_path, parse_output)?,
        None => println!("{}", parse_output),
    };

    Ok(())
}
