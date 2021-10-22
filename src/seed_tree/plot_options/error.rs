use crate::seed_tree::sha1_string::Sha1String;
use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq)]
pub enum PlotOptionError {
    MultiplePredecessorsNotSupported(HashSet<Sha1String>),
}
