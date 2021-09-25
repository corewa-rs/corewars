//! This module is used for parsing a Redcode program.
//! It operates in multiple phases, which are found in the [phase](phase/index.html)
//! module. Each phase passes its result to the next phase.

// TODO(#43)
#![allow(clippy::missing_panics_doc)]

pub use error::{Error, Warning};
pub use result::Result;

mod error;
mod grammar;
mod phase;
mod result;

use std::convert::TryFrom;

use corewars_core::load_file::Warrior;

use phase::{CommentsRemoved, Evaluated, Expanded, Output, Phase, Raw};

/// Parse a given input string into a [`Result`](Result). If successful the
/// `Result` will contain a `Warrior`, otherwise it will contain an error. In
/// either case, one or more [`Warning`](error::Warning)s may be generated with
/// the `Warrior`.
pub fn parse(input: &str) -> Result<Warrior> {
    parse_impl(input).into()
}

fn parse_impl(input: &str) -> std::result::Result<Warrior, Error> {
    let raw = Phase::<Raw>::from(input);

    let cleaned = Phase::<CommentsRemoved>::from(raw);

    let expanded = Phase::<Expanded>::from(cleaned);

    let evaluated = Phase::<Evaluated>::try_from(expanded)?;

    let output = Phase::<Output>::from(evaluated);

    Ok(output.state.warrior)
}
