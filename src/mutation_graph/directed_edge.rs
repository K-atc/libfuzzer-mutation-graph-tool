use super::sha1_string::Sha1String;

use std::hash::{Hash, Hasher};
use crate::mutation_graph::mutation_graph_edge::MutationGraphEdge;

#[derive(Debug, Clone)]
pub struct DirectedEdge {
    pub(crate) parent: Sha1String,
    pub(crate) child: Sha1String,
}

impl Hash for DirectedEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parent.hash(state);
        self.child.hash(state);
    }
}

impl PartialEq for DirectedEdge {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.child == other.child
    }
}

impl Eq for DirectedEdge {}

impl DirectedEdge {
    pub fn new(edge: &MutationGraphEdge) -> Self {
        Self {
            parent: edge.parent.clone(),
            child: edge.child.clone(),
        }
    }
}