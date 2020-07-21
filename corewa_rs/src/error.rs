use std::error;
use std::fmt;
use std::string::ToString;

#[derive(Debug, PartialEq)]
pub struct Error {
    pub details: String,
}

impl error::Error for Error {}

impl Error {
    pub fn new<S: ToString>(details: S) -> Self {
        Self {
            details: details.to_string(),
        }
    }

    pub fn no_input() -> Error {
        Error {
            details: "No input found".to_owned(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.details)
    }
}
pub trait IntoError: fmt::Display {}

impl<T: pest::RuleType> IntoError for pest::error::Error<T> {}
impl IntoError for String {}
impl IntoError for &str {}
impl IntoError for std::num::TryFromIntError {}

impl<T: IntoError> From<T> for Error {
    fn from(displayable_error: T) -> Self {
        Error {
            details: displayable_error.to_string(),
        }
    }
}
