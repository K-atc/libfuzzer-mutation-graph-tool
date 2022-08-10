use super::node_name::NodeName;
use std::cmp::Ordering;

#[derive(Debug, Clone, Hash)]
pub struct MutationGraphEdge {
    pub parent: NodeName,
    pub child: NodeName,
    pub label: String,
}

impl PartialEq for MutationGraphEdge {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.child == other.child
    }
}

impl Eq for MutationGraphEdge {}

impl PartialOrd for MutationGraphEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MutationGraphEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.parent.cmp(&other.parent).is_eq() {
            self.child.cmp(&other.child)
        } else {
            Ordering::Equal
        }
    }
}
