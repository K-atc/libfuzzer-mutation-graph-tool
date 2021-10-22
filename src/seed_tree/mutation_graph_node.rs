use super::sha1_string::Sha1String;
use std::cmp::Ordering;

#[derive(Debug, Clone, Hash)]
pub struct MutationGraphNode {
    pub sha1: Sha1String,
    pub crashed: bool,
}

impl PartialEq for MutationGraphNode {
    fn eq(&self, other: &Self) -> bool {
        self.sha1 == other.sha1
    }
}

impl Eq for MutationGraphNode {}

impl Ord for MutationGraphNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sha1.cmp(&other.sha1)
    }
}

impl PartialOrd for MutationGraphNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl MutationGraphNode {
    pub fn new(sha1: &Sha1String) -> Self {
        Self { sha1: sha1.clone(), crashed: false }
    }
    pub fn new_with_metadata(sha1: &Sha1String, crashed: bool) -> Self {
        Self { sha1: sha1.clone(), crashed }
    }
}
