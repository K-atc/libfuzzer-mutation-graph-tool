#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    RegexError(regex::Error),
    UnknownLine(String),
    SyntaxError(&'static str, String),
}
