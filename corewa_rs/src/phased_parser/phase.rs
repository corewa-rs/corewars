mod comment;

use comment::Info;

/// The data type that is passed through the parser phases. This is essentially
/// a simple state machine, which transitions to the next state by passing through
/// a parser phase.
#[derive(Debug)]
pub enum Buffer {
    Raw(String),
    NoComments {
        /// The parsed out metadata for the program
        info: comment::Info,
        /// The remainder of the program after comments have been removed
        remaining: String,
    },
}

impl Buffer {
    fn remove_comments(self) -> Self {
        if let Buffer::Raw(remaining) = self {
            comment::Info::extract_from_string(remaining)
        } else {
            self
        }
    }
}
