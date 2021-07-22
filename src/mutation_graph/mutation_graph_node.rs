use super::sha1_string::Sha1String;

#[derive(Debug, Clone, Hash)]
pub struct MutationGraphNode {
    pub sha1: Sha1String,
}

impl PartialEq for MutationGraphNode {
    fn eq(&self, other: &Self) -> bool {
        self.sha1 == other.sha1
    }
}

impl Eq for MutationGraphNode {}

impl MutationGraphNode {
    pub fn new(sha1: Sha1String) -> Self {
        Self {
            sha1,
        }
    }
}