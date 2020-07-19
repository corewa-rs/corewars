use std::{
    error::Error,
    fs,
    io::{self, Read},
    path::PathBuf,
};

use lazy_static::lazy_static;
use structopt::StructOpt;

use crate::parser::parse;

lazy_static! {
    static ref IO_SENTINEL: PathBuf = PathBuf::from("-");
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
/// Parse, assemble, and save Redcode files
struct CliOptions {
    /// Input file; use "-" to read from stdin
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Save/print a program in "load file" format
    #[structopt(name = "dump")]
    Dump {
        /// Output file; defaults to stdout ("-")
        #[structopt(long, short, parse(from_os_str), default_value = IO_SENTINEL.to_str().unwrap())]
        output_file: PathBuf,

        /// Whether labels, expressions, macros, etc. should be resolved and
        /// expanded in the output
        #[structopt(long, short = "E")]
        no_expand: bool,
    },
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli_options = CliOptions::from_args();

    let mut input = String::new();

    if cli_options.input_file == *IO_SENTINEL {
        io::stdin().read_to_string(&mut input)?;
    } else {
        input = fs::read_to_string(cli_options.input_file)?;
    }

    let parsed_core = parse(input.as_str())?;

    // TODO bring back warnings

    match cli_options.command {
        Command::Dump {
            output_file,
            no_expand,
        } => {
            if no_expand {
                unimplemented!()
            }

            if output_file == *IO_SENTINEL {
                println!("{}", parsed_core);
            } else {
                fs::write(output_file, format!("{}\n", parsed_core))?;
            };
        }
    };

    Ok(())
}
