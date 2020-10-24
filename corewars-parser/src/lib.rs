//! This module is used for parsing a Redcode program.
//! It operates in multiple phases, which are found in the [phase](phase/index.html)
//! module. Each phase passes its result to the next phase.

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

/// Parse a warrior inline (no strings needed) and return the result. of `parse`.
/// You may find it helpful to write comments with `; //` for syntax highlighting.
///
/// # Examples
///
/// ```
/// use corewars_parser::inline_warrior;
///
/// let warrior = inline_warrior! {
///     begin:  add # 1, @ 2 ;// okay hmm doesn't work because newlines
///     sub 3, 4
/// };
///
/// dbg!(&warrior);
/// assert_eq!(warrior.unwrap().len(), 2);
/// ```
// TODO make a proc-macro like `inline_python` so it can fail at compile time
// https://blog.m-ou.se/writing-python-inside-rust-1/ for details on how
#[macro_export]
macro_rules! inline_warrior {
    ($($code:tt)*) => {
        $crate::parse(stringify!($($code)*));
    }
}
