//! This module defines the phased_parser state machine. Each phase of the parser
//! is a submodule within this module.

pub mod comment;

use std::{convert::Infallible, str::FromStr};

/// The data type that is passed through the parser phases. This is a simple state
/// machine, which transitions to the next state by passing through a parser phase.
#[derive(Debug)]
pub struct Buffer<S> {
    contents: String,
    state: S,
}

impl FromStr for Buffer<Raw> {
    type Err = Infallible;

    fn from_str(buf: &str) -> Result<Self, Infallible> {
        Ok(Buffer {
            contents: buf.to_string(),
            state: Raw,
        })
    }
}

/// The initial state of [Buffer](struct.Buffer.html), before any preprocessing has occurred.
pub struct Raw;

// TODO: Need to consider TryFrom instead of From? Some transitions could fail
impl From<Buffer<Raw>> for Buffer<comment::Stripped> {
    fn from(b: Buffer<Raw>) -> Self {
        let state = comment::Info::extract_from_string(&b.contents);
        Self {
            contents: b.contents,
            state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use textwrap_macros::dedent;

    #[test]
    fn transition_strip_comments() {
        let buf = Buffer::<Raw>::from_str(
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

        let stripped_buf = Buffer::<comment::Stripped>::from(buf);

        assert_eq!(stripped_buf.state.lines, vec![String::from("ORG 123")]);
    }
}
