use crate::seed_tree::node_name::NodeName;

#[derive(Debug, Eq, PartialEq)]
pub enum MutationGraphError {
    NodeNotExists(NodeName),
    FmtError(std::fmt::Error),
    // IoError, // NOTE: std::io::Error does not satisfies PartialEq
}
