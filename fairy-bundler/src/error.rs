use std::{error::Error as StdError, fmt, io};
use swc_ecma_parser::error::Error as SwcParserError;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] ParserError),
}

#[derive(Debug)]
pub struct ParserError {
    inner: SwcParserError,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.kind().msg())
    }
}

impl StdError for ParserError {}

impl From<SwcParserError> for ParserError {
    fn from(inner: SwcParserError) -> Self {
        ParserError { inner }
    }
}

impl From<SwcParserError> for Error {
    fn from(e: SwcParserError) -> Self {
        Error::Parse(e.into())
    }
}
