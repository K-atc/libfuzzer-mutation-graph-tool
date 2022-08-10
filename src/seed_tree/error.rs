use crate::seed_tree::node_name::NodeName;

use super::file_hash::FileHash;

#[derive(Debug, Eq, PartialEq)]
pub enum MutationGraphError {
    NodeNotExists(NodeName),
    FileHashNotExists(FileHash),
    FmtError(std::fmt::Error),
    // IoError, // NOTE: std::io::Error does not satisfies PartialEq
}
