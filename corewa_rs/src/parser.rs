//! This module is used for parsing a Redcode program.
//! It operates in multiple phases, which are found in the [phase](phase/index.html)
//! module. Each phase passes its result to the next phase.

use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use err_derive::Error;

mod grammar;
mod phase;

use crate::load_file;
use phase::{Clean, Deserialized, Expand, Phase, Raw};

/// The main error type that may be returned by the parser.
#[derive(Debug, Error)]
pub enum ParseError {}

pub struct Warrior {
    pub core: load_file::Instructions,
}

impl fmt::Display for Warrior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: metadata
        write!(
            f,
            "{}",
            self.core.iter().fold(String::new(), |result, instruction| {
                result + &instruction.to_string() + "\n"
            })
        )
    }
}

// TODO: function for parsing without expansion?

pub fn parse(input: &str) -> Result<Warrior, Box<dyn Error>> {
    // UNWRAP: Infallible conversion
    let raw = Phase::<Raw>::from_str(input).unwrap();

    let cleaned = Phase::<Clean>::from(raw);

    let expanded = Phase::<Expand>::from(cleaned);

    let deserialized = Phase::<Deserialized>::try_from(expanded)?;

    Ok(Warrior {
        core: deserialized.state.instructions,
    })
}
