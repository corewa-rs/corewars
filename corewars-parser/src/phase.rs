//! This module defines the parser state machine. Each phase of the parser
//! is a submodule within this module.

use std::convert::TryFrom;

mod comment;
mod evaluation;
mod expansion;

use corewars_core::load_file;

use super::error::Error;

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

impl From<&str> for Phase<Raw> {
    fn from(buf: &str) -> Self {
        Phase {
            buffer: buf.to_string(),
            state: Raw,
        }
    }
}

/// The Phase after comments have been removed and metadata parsed from comments.
/// This phase also parses ORG and END, and removes any text after END
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CommentsRemoved {
    pub lines: Vec<String>,
    pub metadata: load_file::Metadata,
    pub origin: Option<String>,
}

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
#[derive(Debug, Default)]
pub struct Expanded {
    /// The expanded lines of text to be parsed later
    lines: Vec<String>,

    /// Metadata gathered in previous phase
    metadata: load_file::Metadata,

    /// The entrypoint to the program, gathered in previous phase. This is still
    /// a string because it may be an expression to be evaluated later
    origin: Option<String>,
}

impl From<Phase<CommentsRemoved>> for Phase<Expanded> {
    fn from(prev: Phase<CommentsRemoved>) -> Self {
        let lines = expansion::expand(prev.state.lines, prev.state.origin);

        Self {
            buffer: prev.buffer,
            state: Expanded {
                lines: lines.text,
                origin: lines.origin,
                metadata: prev.state.metadata,
            },
        }
    }
}

/// The program after all expressions have been evaluated. This stage handles
/// arithmetic and boolean logic, as well as parsing regular integer values.
#[derive(Debug, Default)]
pub struct Evaluated {
    /// Metadata gathered in previous phase
    metadata: load_file::Metadata,

    /// The parsed program
    program: load_file::Program,
}

impl TryFrom<Phase<Expanded>> for Phase<Evaluated> {
    type Error = Error;

    fn try_from(prev: Phase<Expanded>) -> Result<Self, Error> {
        let instructions = evaluation::evaluate(prev.state.lines)?;
        let origin = prev
            .state
            .origin
            .as_ref()
            .map(|s| evaluation::evaluate_expression(s))
            .transpose()?;

        // TODO evaluate assertions

        Ok(Self {
            buffer: prev.buffer,
            state: Evaluated {
                metadata: prev.state.metadata,
                program: load_file::Program {
                    instructions,
                    origin,
                },
            },
        })
    }
}

/// The final resulting output of the parser, which is suitable for simulation.
#[derive(Debug)]
pub struct Output {
    pub warrior: load_file::Warrior,
}

impl From<Phase<Evaluated>> for Phase<Output> {
    fn from(prev: Phase<Evaluated>) -> Self {
        Self {
            buffer: prev.buffer,
            state: Output {
                warrior: load_file::Warrior {
                    metadata: prev.state.metadata,
                    program: prev.state.program,
                },
            },
        }
    }
}
