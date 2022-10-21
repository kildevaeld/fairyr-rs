use std::io;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("compiler error: {0}")]
    Compiler(#[from] anyhow::Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
