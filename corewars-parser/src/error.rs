//! Error types for the corewars library

use std::num::TryFromIntError;

use thiserror::Error as ThisError;

use corewars_core::load_file::Opcode;

// TODO: use pest spans for error reporting? Or at least line number

/// An error that occurred while parsing a warrior.
#[derive(ThisError, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The warrior contained a reference to a label that doesn't exist.
    #[error("no such label {label:?}")]
    LabelNotFound { label: String, line: Option<usize> },

    /// An invalid warrior origin (not a positive integer) was specified.
    #[error("invalid origin specified")]
    InvalidOrigin(#[from] TryFromIntError),

    /// The input string was ill-formed Redcode syntax.
    #[error("invalid syntax")]
    InvalidSyntax(#[from] super::grammar::SyntaxError),

    /// The given opcode was not given enough arguments.
    #[error("expected additional arguments for {opcode} opcode")]
    InvalidArguments { opcode: Opcode },
}

/// A warning that occurred while parsing a warrior.
#[derive(ThisError, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Warning {
    /// Attempt to define the warrior origin more than once.
    #[error("origin already defined as {old:?}, new definition {new:?} will be ignored")]
    OriginRedefinition { old: String, new: String },

    /// Empty EQU substitution.
    #[error("right-hand side of substitution for label {0:?} is empty")]
    EmptySubstitution(String),

    /// Offset label declaration with no instruction.
    #[error("no instruction offset for label {0:?}, it will not b")]
    EmptyOffset(String),
}
