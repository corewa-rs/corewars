//! This module is used for parsing a Redcode program.
//! It operates in multiple phases, which are found in the `phase`
//! module. Each phase passes its result to the next phase.

use std::error;

use err_derive::Error;

use crate::load_file::*;

mod phase;

/// The main error type that may be returned by the parser.
#[derive(Debug, Error)]
pub enum ParseError {}

pub fn parse() -> Result<Program, ParseError> {
    todo!()
}
