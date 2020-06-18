// Extern crates
extern crate itertools;
extern crate lazy_static;
extern crate pest;
extern crate pest_derive;
extern crate regex;
extern crate structopt;

// Test-only extern crates
#[cfg(any(test, feature = "integration_test"))]
pub extern crate predicates;
#[cfg(any(test, feature = "integration_test"))]
pub extern crate predicates_tree;

// Macro-exporting modules
#[macro_use]
mod util;

// Macro-exporting test-only modules
#[cfg(any(test, feature = "integration_test"))]
#[macro_use]
pub mod testutil;

// Public modules
pub mod cli;

// Exported functions
pub use parser::parse;

// Private modules
mod load_file;
mod parser;
mod phased_parser;
