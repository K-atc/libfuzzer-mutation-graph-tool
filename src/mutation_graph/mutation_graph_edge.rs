use super::sha1_string::Sha1String;

#[derive(Debug, Clone, Hash)]
pub struct MutationGraphEdge {
    pub parent: Sha1String,
    pub child: Sha1String,
    pub label: String,
}

impl PartialEq for MutationGraphEdge {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.child == other.child
    }
}

impl Eq for MutationGraphEdge {}