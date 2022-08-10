use super::node_name::NodeName;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Hash, Default)]
pub struct MutationGraphNode {
    pub name: NodeName,
    pub crashed: bool,
    pub file: PathBuf,
}

impl PartialEq for MutationGraphNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for MutationGraphNode {}

impl Ord for MutationGraphNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for MutationGraphNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl MutationGraphNode {
    pub fn new(sha1: &NodeName) -> Self {
        Self {
            name: sha1.clone(),
            ..Default::default()
        }
    }

    pub fn new_with_metadata(sha1: &NodeName, crashed: bool, file: &Path) -> Self {
        Self {
            name: sha1.clone(),
            crashed,
            file: file.to_path_buf(),
        }
    }
}
