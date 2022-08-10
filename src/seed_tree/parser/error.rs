use crate::seed_tree::error::MutationGraphError;

use std::io;
use std::path::PathBuf;

type ErrorMessage = &'static str;

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    RegexError(regex::Error),
    UnknownLine(String),
    SyntaxError(ErrorMessage, String),
    UnexpectedFilePath(PathBuf),
    UnexpectedDirectoryPath(PathBuf),
    StringEncoding,
    MutationGraph(MutationGraphError),
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<regex::Error> for ParseError {
    fn from(error: regex::Error) -> Self {
        Self::RegexError(error)
    }
}
