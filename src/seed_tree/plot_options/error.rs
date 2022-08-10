use crate::seed_tree::node_name::NodeName;
use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq)]
pub enum PlotOptionError {
    MultiplePredecessorsNotSupported(HashSet<NodeName>),
}
