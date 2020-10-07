//! Error handling for the corewars parser.
//! [`Result`](Result) matches the `std::result::Result` type, except that it
//! may also contain warnings alongside either an `Ok` or `Err` type.

use super::error::{Error, Warning};

use std::result::Result as StdResult;

/// `Result` mimics the `std::result::Result` type, but each variant also carries
/// zero or more [`Warning`](Warning)s with it.
#[must_use = "this `Result` may be an `Err` variant, which should be handled"]
pub enum Result<T> {
    /// Contains the success value and zero or more warnings
    Ok(T, Vec<Warning>),

    /// Contains the error value and zero or more warnings
    Err(Error, Vec<Warning>),
}

impl<T> Result<T> {
    /// Create an `Ok` variant from a value.
    pub fn ok(value: T) -> Self {
        Self::Ok(value, Vec::new())
    }

    /// Create an `Err` variant from an error.
    pub fn err(err: Error) -> Self {
        Self::Err(err, Vec::new())
    }
}

impl<T> From<StdResult<T, Error>> for Result<T> {
    fn from(result: StdResult<T, Error>) -> Self {
        match result {
            Ok(value) => Self::Ok(value, Vec::new()),
            Err(err) => Self::Err(err, Vec::new()),
        }
    }
}

impl<T> From<Error> for Result<T> {
    fn from(err: Error) -> Self {
        Self::Err(err, Vec::new())
    }
}

// TODO some more impls. Probably some kind of deref coercion to make it more ergo
