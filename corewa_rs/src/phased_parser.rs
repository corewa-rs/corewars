//! This module is used for parsing a Redcode program.
//! It operates in multiple phases, which are found in the [phase](phase/index.html)
//! module. Each phase passes its result to the next phase.

mod phase;

use std::str::FromStr;

use err_derive::Error;

use crate::load_file::*;
use phase::{Cleaned, Phase, Raw};

/// The main error type that may be returned by the parser.
#[derive(Debug, Error)]
pub enum ParseError {}

pub fn parse(input: &str) -> Result<Program, ParseError> {
    // UNWRAP: Infallible conversion
    let raw = Phase::<Raw>::from_str(input).unwrap();

    let _cleaned = Phase::<Cleaned>::from(raw);

    todo!()
}
