//! This module defines the phased_parser state machine. Each phase of the parser
//! is a submodule within this module.

pub mod clean;

use std::{convert::Infallible, str::FromStr};

/// The data type that is passed through the parser phases. This is a simple state
/// machine, which transitions to the next state by passing through a parser phase.
#[derive(Debug)]
pub struct Phase<S> {
    /// The original input to the parser, which can be used for spans / string views
    buffer: String,
    /// State specific to the current phase of the state machine
    state: S,
}

impl FromStr for Phase<Raw> {
    type Err = Infallible;

    fn from_str(buf: &str) -> Result<Self, Infallible> {
        Ok(Phase {
            buffer: buf.to_string(),
            state: Raw,
        })
    }
}

/// The initial state of [Buffer](struct.Buffer.html), before any preprocessing has occurred.
pub struct Raw;

// TODO: Need to consider TryFrom instead of From? Some transitions could fail
impl From<Phase<Raw>> for Phase<Cleaned> {
    fn from(b: Phase<Raw>) -> Self {
        let state = clean::Info::extract_from_string(&b.buffer);
        Self {
            buffer: b.buffer,
            state,
        }
    }
}

/// The state of Phase after cleans have been removed and metadata parsed from
/// the cleans. Any text after END is also removed.
pub struct Cleaned {
    pub lines: Vec<String>,
    pub metadata: clean::Info,
}

#[cfg(test)]
mod tests {
    use super::*;
    use textwrap_macros::dedent;

    #[test]
    fn transition_clean() {
        let raw_phase = Phase::<Raw>::from_str(
            dedent!(
                "
                ;redcode
                ; author Ian Chamberlain
                ORG 123 ; begin here
                "
            )
            .trim(),
        )
        .unwrap();

        let stripped_phase = Phase::<Cleaned>::from(raw_phase);

        assert_eq!(stripped_phase.state.lines, vec![String::from("ORG 123")]);
    }
}
