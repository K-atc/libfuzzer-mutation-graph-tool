use crate::seed_tree::error::MutationGraphError;

pub type Result<T> = std::result::Result<T, MutationGraphError>;
