//! This module is used for parsing a Redcode program.
//! It operates in multiple phases, which are found in the [phase](phase/index.html)
//! module. Each phase passes its result to the next phase.

mod grammar;
mod phase;

use std::convert::TryFrom;
use std::error::Error;
use std::str::FromStr;

use err_derive::Error;

use crate::load_file::Warrior;
use phase::{CommentsRemoved, Evaluated, Expanded, Output, Phase, Raw};

/// The main error type that may be returned by the parser.
#[derive(Debug, Error)]
pub enum ParseError {}

// TODO: function for parsing without expansion

/// Parse a given input string into a
pub fn parse(input: &str) -> Result<Warrior, Box<dyn Error>> {
    // Unwrap: infallible conversion
    let raw = Phase::<Raw>::from_str(input).unwrap();

    let cleaned = Phase::<CommentsRemoved>::from(raw);

    let expanded = Phase::<Expanded>::from(cleaned);

    let evaluated = Phase::<Evaluated>::try_from(expanded)?;

    let output = Phase::<Output>::try_from(evaluated)?;

    Ok(output.state.warrior)
}
