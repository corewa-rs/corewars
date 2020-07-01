// Extern crates
extern crate itertools;
extern crate lazy_static;
extern crate pest;
extern crate pest_derive;
extern crate regex;
extern crate structopt;

// Macro-exporting modules
#[macro_use]
mod util;

// Public modules
pub mod cli;

// Exported functions
pub use parser::parse;

// Private modules
mod load_file;
mod parser;
mod phased_parser;
