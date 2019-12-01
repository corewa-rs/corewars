use std::{error::Error, fs, path::PathBuf};

use lazy_static::lazy_static;
use structopt::StructOpt;

use crate::parser;

lazy_static! {
    static ref STDOUT: PathBuf = PathBuf::from("-");
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
/// Parse, assemble, and save Redcode files
struct CliOptions {
    /// Input file
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Save/print a program in 'load file' format
    #[structopt(name = "dump")]
    Dump {
        /// Output file - defaults to stdout ('-')
        #[structopt(long, short, parse(from_os_str), default_value = STDOUT.to_str().unwrap())]
        output_file: PathBuf,

        /// Whether labels, expressions, macros, etc. should be resolved and
        /// expanded in the output
        #[structopt(long, short = "E")]
        no_expand: bool,
    },
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli_options = CliOptions::from_args();

    let input_program = fs::read_to_string(cli_options.input_file)?;

    let mut parsed_core = parser::parse(input_program.as_str())?;

    match cli_options.command {
        Command::Dump {
            output_file,
            no_expand,
        } => {
            if !no_expand {
                parsed_core.resolve()?;
            }

            if output_file == *STDOUT {
                println!("{}", parsed_core);
            } else {
                fs::write(output_file, format!("{}", parsed_core))?;
            };
        }
    };

    Ok(())
}
