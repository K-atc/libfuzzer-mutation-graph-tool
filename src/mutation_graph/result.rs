use crate::mutation_graph::error::MutationGraphError;

pub type Result<T> = std::result::Result<T, MutationGraphError>;
