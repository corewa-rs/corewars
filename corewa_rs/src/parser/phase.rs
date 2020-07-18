//! This module defines the parser state machine. Each phase of the parser
//! is a submodule within this module.

use std::convert::{Infallible, TryFrom};
use std::str::FromStr;

mod comment;
mod deserialize;
mod expansion;

use crate::error::Error;
use crate::load_file;

/// The data type that is passed through the parser phases. This is a simple state
/// machine, which transitions to the next state by passing through a parser phase.
#[derive(Debug)]
pub struct Phase<PhaseState> {
    /// The original input to the parser, which can be used for spans / string views
    buffer: String,
    /// State specific to the current phase of the state machine
    pub state: PhaseState,
}

/// The initial state of parsing, before any preprocessing has occurred.
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
#[derive(Debug, Default, PartialEq)]
pub struct CommentsRemoved {
    pub lines: Vec<String>,
    pub metadata: load_file::Metadata,
    pub origin: Option<String>,
}

// TODO: Need to consider TryFrom instead of From? Some transitions could fail
impl From<Phase<Raw>> for Phase<CommentsRemoved> {
    fn from(prev: Phase<Raw>) -> Self {
        let state = comment::extract_from_string(&prev.buffer);
        Self {
            buffer: prev.buffer,
            state,
        }
    }
}

/// The phase in which labels are collected and expanded. Resulting struct
/// contains metadata from previous phase and the expanded lines
#[derive(Debug)]
pub struct Expanded {
    lines: Vec<String>,
    metadata: load_file::Metadata,
}

impl From<Phase<CommentsRemoved>> for Phase<Expanded> {
    fn from(prev: Phase<CommentsRemoved>) -> Self {
        let lines = expansion::expand(prev.state.lines);

        Self {
            buffer: prev.buffer,
            state: Expanded {
                lines,
                metadata: prev.state.metadata,
            },
        }
    }
}

/// The phase in which string-based lines are converted into in-memory data structures
/// for later simulation. This should be the final phase of parsing.
#[derive(Debug)]
pub struct Deserialized {
    pub warrior: load_file::Warrior,
}

impl TryFrom<Phase<Expanded>> for Phase<Deserialized> {
    type Error = Error;

    fn try_from(prev: Phase<Expanded>) -> Result<Self, Error> {
        let program = deserialize::deserialize(prev.state.lines)?;

        Ok(Self {
            buffer: prev.buffer,
            state: Deserialized {
                warrior: load_file::Warrior {
                    metadata: prev.state.metadata,
                    program,
                },
            },
        })
    }
}
