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

use std::collections::hash_map::Values;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct MutationGraph {
    // Stores real data
    node: HashMap<Sha1String, MutationGraphNode>,
    edge: HashMap<DirectedEdge, MutationGraphEdge>,
    weak_edge: HashMap<DirectedEdge, MutationGraphEdge>,

    // Indexes to search nodes
    children: HashMap<Sha1String, HashSet<Sha1String>>,
    parent: HashMap<Sha1String, Sha1String>,
}

impl MutationGraph {
    pub fn new() -> Self {
        Self {
            node: HashMap::new(),
            edge: HashMap::new(),
            weak_edge: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
        }
    }

    pub fn nodes(&self) -> Values<Sha1String, MutationGraphNode> {
        self.node.values()
    }

    pub fn add_node(&mut self, node: &MutationGraphNode) -> () {
        self.node.insert(node.sha1.clone(), node.clone());
        self.children.insert(node.sha1.clone(), HashSet::new());
    }

    pub fn add_edge(&mut self, edge: &MutationGraphEdge) -> () {
        // Some times explicit node declarations are missed in original mutation graph node
        if self.get_node(&edge.parent).is_none() {
            self.add_node(&MutationGraphNode::new(&edge.parent))
        }
        if self.get_node(&edge.child).is_none() {
            self.add_node(&MutationGraphNode::new(&edge.child))
        }

        // Insert edge and update indexes avoiding making closed chains
        if self.root_of(&edge.parent) != self.root_of(&edge.child) {
            self.edge.insert(DirectedEdge::new(&edge), edge.clone());

            match self.children.get_mut(&edge.parent) {
                Some(key) => {
                    key.insert(edge.child.clone());
                    ()
                }
                None => {
                    self.children.insert(
                        edge.parent.clone(),
                        HashSet::from_iter([edge.child.clone()].iter().cloned()),
                    );
                    ()
                }
            };

            self.parent.insert(edge.child.clone(), edge.parent.clone());
        } else {
            self.weak_edge
                .insert(DirectedEdge::new(&edge), edge.clone());
        }
    }

    pub fn get_node(&self, sha1: &Sha1String) -> Option<&MutationGraphNode> {
        self.node.get(sha1)
    }

    pub fn children_of(&self, parent: &Sha1String) -> Option<&HashSet<Sha1String>> {
        self.children.get(parent)
    }

    pub fn parent_of(&self, child: &Sha1String) -> Option<&Sha1String> {
        self.parent.get(child)
    }

    pub fn root_of<'a>(&'a self, node: &'a Sha1String) -> Result<&'a Sha1String> {
        if self.get_node(node).is_none() {
            return Err(MutationGraphError::NodeNotExists(node.clone()));
        }
        match self.parent_of(node) {
            Some(parent) => self.root_of(parent),
            None => Ok(node),
        }
    }

    pub fn predecessors_of(&self, node: &Sha1String) -> Result<Vec<&Sha1String>> {
        if self.get_node(node).is_none() {
            return Err(MutationGraphError::NodeNotExists(node.clone()));
        }
        match self.parent_of(node) {
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

    pub fn leaves(&self) -> HashSet<&Sha1String> {
        self.children
            .iter()
            .filter(|(_, v)| v.len() == 0)
            .map(|(k, _)| k)
            .collect()
    }

    pub fn roots(&self) -> HashSet<&Sha1String> {
        self.node
            .keys()
            .filter(|v| self.parent_of(v).is_none())
            .collect()
    }

    // Helper function
    fn _dot_graph(&self) -> std::result::Result<String, std::fmt::Error> {
        let mut res = String::new();
        write!(&mut res, "digraph {{\n")?;
        for node in self.node.values() {
            write!(&mut res, "\"{}\"\n", node.sha1)?;
        }
        for edge in self.edge.values() {
            write!(
                &mut res,
                "\"{}\" -> \"{}\" [label=\"{}\", splines=\"curved\"];\n",
                edge.parent, edge.child, edge.label
            )?;
        }
        for weak_edge in self.weak_edge.values() {
            write!(
                &mut res,
                "\"{}\" -> \"{}\" [label=\"{}\", style=\"dashed\"];\n",
                weak_edge.parent, weak_edge.child, weak_edge.label
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
    use std::collections::HashSet;
    use std::iter::FromIterator;

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

        println!("[*] graph = {:#?}", graph);

        assert_eq!(graph.parent_of(&node_1_sha1), None);
        assert_eq!(graph.parent_of(&node_2_sha1), Some(&node_1_sha1));
        assert_eq!(graph.parent_of(&node_3_sha1), Some(&node_1_sha1));

        assert_eq!(graph.root_of(&node_1_sha1), Ok(&node_1_sha1));
        assert_eq!(graph.root_of(&node_4_sha1), Ok(&node_1_sha1));
        assert_eq!(
            graph.root_of(&no_such_node_sha1),
            Err(MutationGraphError::NodeNotExists(no_such_node_sha1.clone()))
        );

        assert_eq!(
            graph.predecessors_of(&node_5_sha1),
            Ok(vec![&node_1_sha1, &node_3_sha1, &node_4_sha1])
        );

        assert_eq!(
            graph.leaves(),
            HashSet::from_iter(vec![&node_2_sha1, &node_5_sha1])
        );

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_sha1]));
    }

    #[test]
    fn test_mutation_graph_missing_explicit_node_decl() {
        let node_1_sha1 = String::from("node_1");
        let node_2_sha1 = String::from("node_2");
        let node_3_sha1 = String::from("node_3");

        let mut graph = MutationGraph::new();
        /*
           (1)
           / \
         (2) (3)
        */
        graph.add_edge(&MutationGraphEdge::new(&node_1_sha1, &node_2_sha1));
        graph.add_edge(&MutationGraphEdge::new(&node_1_sha1, &node_3_sha1));

        assert_eq!(
            graph
                .nodes()
                .map(|v| &v.sha1)
                .collect::<HashSet<&Sha1String>>(),
            HashSet::from_iter([&node_1_sha1, &node_2_sha1, &node_3_sha1])
        );

        assert_eq!(
            graph.leaves(),
            HashSet::from_iter(vec![&node_2_sha1, &node_3_sha1])
        );

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_sha1]));
    }

    #[test]
    fn test_mutation_graph_cycle_graph() {
        let node_1_sha1 = String::from("node_1");
        let node_2_sha1 = String::from("node_2");
        let node_3_sha1 = String::from("node_3");

        let mut graph = MutationGraph::new();
        /*
           (1)
           / \
         (2)-(3)
        */
        graph.add_edge(&MutationGraphEdge::new(&node_1_sha1, &node_2_sha1));
        graph.add_edge(&MutationGraphEdge::new(&node_2_sha1, &node_3_sha1));
        graph.add_edge(&MutationGraphEdge::new(&node_3_sha1, &node_1_sha1));

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_sha1]));
    }
}
