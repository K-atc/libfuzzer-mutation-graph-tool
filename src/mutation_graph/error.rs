use crate::mutation_graph::sha1_string::Sha1String;

#[derive(Debug, Eq, PartialEq)]
pub enum MutationGraphError {
    NodeNotExists(Sha1String),
    FmtError(std::fmt::Error),
    // IoError, // NOTE: std::io::Error does not satisfies PartialEq
}
