use crate::seed_tree::error::MutationGraphError;
use std::path::PathBuf;

type ErrorMessage = &'static str;

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    RegexError(regex::Error),
    UnknownLine(String),
    SyntaxError(ErrorMessage, String),
    UnexpectedFilePath(PathBuf),
    UnexpectedDirectoryPath(PathBuf),
    StringEncoding,
    MutationGraph(MutationGraphError),
}
