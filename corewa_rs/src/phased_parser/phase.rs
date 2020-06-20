//! This module defines the phased_parser state machine. Each phase of the parser
//! is a submodule within this module.

pub mod clean;
mod expand;

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

/// The initial state of [Buffer](struct.Buffer.html), before any preprocessing has occurred.
pub struct Raw;

impl FromStr for Phase<Raw> {
    type Err = Infallible;

    fn from_str(buf: &str) -> Result<Self, Infallible> {
        Ok(Phase {
            buffer: buf.to_string(),
            state: Raw,
        })
    }
}

/// The Phase after comments have been removed and metadata parsed from comments.
/// This phase also parses ORG and END, and removes any text after END
#[derive(Debug)]
pub struct Clean {
    pub lines: Vec<String>,
    pub metadata: clean::Info,
}

// TODO: Need to consider TryFrom instead of From? Some transitions could fail
impl From<Phase<Raw>> for Phase<Clean> {
    fn from(prev: Phase<Raw>) -> Self {
        let state = clean::Info::extract_from_string(&prev.buffer);
        Self {
            buffer: prev.buffer,
            state,
        }
    }
}

/// The phase in which labels are collected and expanded. Resulting struct
/// contains metadata from previous phase, as well as a table of labels collected.
#[derive(Debug)]
pub struct Expand {
    pub lines: Vec<String>,
    pub labels: expand::Labels,
    pub metadata: clean::Info,
}

impl From<Phase<Clean>> for Phase<Expand> {
    fn from(prev: Phase<Clean>) -> Self {
        let info = expand::Info::collect(prev.state.lines);

        Self {
            buffer: prev.buffer,
            state: Expand {
                lines: info.lines,
                metadata: prev.state.metadata,
                labels: info.labels,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use textwrap_macros::dedent;

    #[test]
    fn transitions() {
        let raw_phase = Phase::<Raw>::from_str(
            dedent!(
                "
                ;redcode
                ; author Ian Chamberlain
                ORG start
                EQU thing 4
                MOV thing, 0
                start
                MOV thing, thing+1

                "
            )
            .trim(),
        )
        .unwrap();

        let _clean_phase = Phase::<Clean>::from(raw_phase);

        // TODO: expansion transition
    }
}
