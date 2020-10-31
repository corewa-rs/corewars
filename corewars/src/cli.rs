use std::{
    error::Error,
    fs,
    io::{self, Read},
    path::PathBuf,
};

use lazy_static::lazy_static;
use structopt::StructOpt;

use corewars_parser as parser;
use corewars_sim::Core;

lazy_static! {
    static ref IO_SENTINEL: PathBuf = PathBuf::from("-");
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
/// Parse, assemble, and save Redcode files
struct CliOptions {
    /// The corewars subcommand to perform
    #[structopt(subcommand)]
    command: Command,

    /// Print additional details while running
    // TODO(#26) hook this up to a log level
    #[structopt(long, short)]
    verbose: bool,

    /// Input file; use "-" to read from stdin
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,
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

    /// Run a warrior to completion
    #[structopt(name = "run")]
    Run {
        /// The max number of cycles to run. Defaults to
        #[structopt(long, short)]
        max_cycles: Option<usize>,
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

    let parsed_core = match parser::parse(input.as_str()) {
        parser::Result::Ok(warrior, warnings) => {
            print_warnings(&warnings);
            Ok(warrior)
        }
        parser::Result::Err(err, warnings) => {
            print_warnings(&warnings);
            Err(err)
        }
    }?;

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
        Command::Run { max_cycles } => {
            let mut core = Core::default();
            core.load_warrior(&parsed_core)?;

            match core.run(max_cycles) {
                Ok(_) => println!(
                    "Warrior stopped after {}max of {} cycles",
                    if max_cycles.is_some() {
                        "specified "
                    } else {
                        ""
                    },
                    core.steps_taken()
                ),
                Err(err) => println!("Warrior failed after {} steps: {}", core.steps_taken(), err),
            }

            if cli_options.verbose {
                println!("Core after execution:\n{}", core);
            }
        }
    };

    Ok(())
}

fn print_warnings(warnings: &[parser::Warning]) {
    for warning in warnings.iter() {
        eprintln!("Warning: {}", warning)
    }
}
