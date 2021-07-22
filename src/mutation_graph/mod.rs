pub mod directed_edge;
pub mod error;
pub mod mutation_graph_edge;
pub mod mutation_graph_node;
pub mod perser;
pub mod result;
pub mod sha1_string;

use directed_edge::DirectedEdge;
use error::MutationGraphError;
use mutation_graph_edge::MutationGraphEdge;
use mutation_graph_node::MutationGraphNode;
use result::Result;
use sha1_string::Sha1String;

use std::collections::{HashMap, HashSet};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct MutationGraph {
    // Stores real data
    node: HashMap<Sha1String, MutationGraphNode>,
    edge: HashMap<DirectedEdge, MutationGraphEdge>,

    // Indexes to search nodes
    children: HashMap<Sha1String, HashSet<Sha1String>>,
    parent: HashMap<Sha1String, Sha1String>,
}

impl MutationGraph {
    pub fn new() -> Self {
        Self {
            node: HashMap::new(),
            edge: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: &MutationGraphNode) -> () {
        self.node.insert(node.sha1.clone(), node.clone());
    }

    pub fn add_edge(&mut self, edge: &MutationGraphEdge) -> () {
        self.edge.insert(DirectedEdge::new(&edge), edge.clone());

        if self.children.get_mut(&edge.parent).is_none() {
            self.children.insert(edge.parent.clone(), HashSet::new());
        }
        match self.children.get_mut(&edge.parent) {
            Some(key) => key.insert(edge.child.clone()),
            None => unreachable!(),
        };

        self.parent.insert(edge.child.clone(), edge.parent.clone());
    }

    pub fn get_node(&self, sha1: &Sha1String) -> Option<&MutationGraphNode> {
        self.node.get(sha1)
    }

    pub fn get_children(&self, parent: &Sha1String) -> Option<&HashSet<Sha1String>> {
        self.children.get(parent)
    }

    pub fn get_parent(&self, child: &Sha1String) -> Option<&Sha1String> {
        self.parent.get(child)
    }

    pub fn get_top<'a>(&'a self, node: &'a Sha1String) -> Result<&'a Sha1String> {
        if self.get_node(node).is_none() {
            return Err(MutationGraphError::NodeNotExists(node.clone()));
        }
        match self.get_parent(node) {
            Some(parent) => self.get_top(parent),
            None => Ok(node),
        }
    }

    pub fn predecessors_of(&self, node: &Sha1String) -> Result<Vec<&Sha1String>> {
        if self.get_node(node).is_none() {
            return Err(MutationGraphError::NodeNotExists(node.clone()));
        }
        match self.get_parent(node) {
            Some(parent) => match self.predecessors_of(parent) {
                Ok(mut res) => {
                    res.push(parent);
                    Ok(res)
                }
                Err(why) => return Err(why),
            },
            None => Ok(vec![]),
        }
    }

    fn _dot_graph(&self) -> std::result::Result<String, std::fmt::Error> {
        let mut res = String::new();
        write!(&mut res, "digraph {{\n")?;
        for node in self.node.values() {
            write!(&mut res, "\"{}\"\n", node.sha1)?;
        }
        for edge in self.edge.values() {
            write!(
                &mut res,
                "\"{}\" -> \"{}\" [label=\"{}\"];\n",
                edge.parent, edge.child, edge.label
            )?;
        }
        write!(&mut res, "}}\n")?;
        Ok(res)
    }

    // Dumps self to dot graph
    pub fn dot_graph(&self) -> Result<String> {
        self._dot_graph().map_err(MutationGraphError::FmtError)
    }
}

#[cfg(test)]
mod test {
    use crate::mutation_graph::error::MutationGraphError;
    use crate::mutation_graph::mutation_graph_edge::MutationGraphEdge;
    use crate::mutation_graph::mutation_graph_node::MutationGraphNode;
    use crate::mutation_graph::sha1_string::Sha1String;
    use crate::mutation_graph::MutationGraph;

    impl MutationGraphEdge {
        pub fn new(parent: &Sha1String, child: &Sha1String) -> Self {
            Self {
                parent: parent.clone(),
                child: child.clone(),
                label: String::from(""),
            }
        }
    }

    #[test]
    fn test_mutation_graph_node() {
        let node_1_sha1 = String::from("node_1");
        let no_such_node_sha1 = String::from("no_such_node");

        let mut graph = MutationGraph::new();

        let node_1 = MutationGraphNode::new(&node_1_sha1);
        graph.add_node(&node_1);

        assert_eq!(graph.get_node(&node_1_sha1), Some(&node_1));
        assert_eq!(graph.get_node(&no_such_node_sha1), None);
    }

    #[test]
    fn test_mutation_graph_edge() {
        let node_1_sha1 = String::from("node_1");
        let node_2_sha1 = String::from("node_2");
        let node_3_sha1 = String::from("node_3");
        let node_4_sha1 = String::from("node_4");
        let node_5_sha1 = String::from("node_5");
        let no_such_node_sha1 = String::from("no_such_node");

        let mut graph = MutationGraph::new();
        /*
           (1)
           / \
         (2) (3)
              |
             (4)
              |
             (5)
        */
        graph.add_node(&MutationGraphNode::new(&node_1_sha1));
        graph.add_node(&MutationGraphNode::new(&node_2_sha1));
        graph.add_node(&MutationGraphNode::new(&node_3_sha1));
        graph.add_node(&MutationGraphNode::new(&node_4_sha1));
        graph.add_node(&MutationGraphNode::new(&node_5_sha1));
        graph.add_edge(&MutationGraphEdge::new(&node_1_sha1, &node_2_sha1));
        graph.add_edge(&MutationGraphEdge::new(&node_1_sha1, &node_3_sha1));
        graph.add_edge(&MutationGraphEdge::new(&node_3_sha1, &node_4_sha1));
        graph.add_edge(&MutationGraphEdge::new(&node_4_sha1, &node_5_sha1));

        assert_eq!(graph.get_parent(&node_1_sha1), None);
        assert_eq!(graph.get_parent(&node_2_sha1), Some(&node_1_sha1));
        assert_eq!(graph.get_parent(&node_3_sha1), Some(&node_1_sha1));

        assert_eq!(graph.get_top(&node_1_sha1), Ok(&node_1_sha1));
        assert_eq!(graph.get_top(&node_4_sha1), Ok(&node_1_sha1));
        assert_eq!(
            graph.get_top(&no_such_node_sha1),
            Err(MutationGraphError::NodeNotExists(no_such_node_sha1.clone()))
        );

        assert_eq!(
            graph.predecessors_of(&node_5_sha1),
            Ok(vec![&node_1_sha1, &node_3_sha1, &node_4_sha1])
        );
    }
}
