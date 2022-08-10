pub mod directed_edge;
pub mod error;
pub mod file_hash;
pub mod mutation_graph_edge;
pub mod mutation_graph_node;
pub mod node_name;
pub mod parser;
pub mod plot_options;
pub mod result;
pub mod util;

use self::directed_edge::DirectedEdge;
use self::error::MutationGraphError;
use self::file_hash::FileHash;
use self::mutation_graph_edge::MutationGraphEdge;
use self::mutation_graph_node::MutationGraphNode;
use self::node_name::NodeName;
use self::plot_options::PlotOptions;
use self::result::Result;

use log::warn;
use std::collections::hash_map::Values;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Write;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct MutationGraph {
    // Stores real data
    node: HashMap<NodeName, MutationGraphNode>,
    edge: HashMap<DirectedEdge, MutationGraphEdge>,
    weak_edge: HashMap<DirectedEdge, MutationGraphEdge>,

    // Indexes to search nodes
    children: HashMap<NodeName, HashSet<NodeName>>,
    parent: HashMap<NodeName, NodeName>,
    file_hash: HashMap<FileHash, NodeName>,
}

impl MutationGraph {
    pub fn new() -> Self {
        Self {
            node: HashMap::new(),
            edge: HashMap::new(),
            weak_edge: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            file_hash: HashMap::new(),
        }
    }

    pub fn nodes(&self) -> Values<NodeName, MutationGraphNode> {
        self.node.values()
    }

    pub fn edges(&self) -> Values<DirectedEdge, MutationGraphEdge> {
        self.edge.values()
    }

    pub fn add_node(&mut self, node: &MutationGraphNode) -> () {
        // NOTE: *Last* inserted node overwhelms existing node when nodes with same name are inserted
        self.node.insert(node.name.clone(), node.clone());
        if !self.children.contains_key(&node.name) {
            // Initialize children on first time
            self.children.insert(node.name.clone(), HashSet::new());
        }
        if !node.hash.is_empty() {
            if let Some(collision_node) =
                self.file_hash.insert(node.hash.clone(), node.name.clone())
            {
                warn!(
                    "Inserted node collides with node name={:?}: node={:?}",
                    collision_node, node
                );
            }
        }
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
            self.edge.insert(DirectedEdge::from(&edge), edge.clone());

            match self.children.get_mut(&edge.parent) {
                Some(children) => {
                    children.insert(edge.child.clone());
                }
                None => {
                    self.children.insert(
                        edge.parent.clone(),
                        HashSet::from_iter([edge.child.clone()].iter().cloned()),
                    );
                }
            };

            self.parent.insert(edge.child.clone(), edge.parent.clone());
        } else {
            self.add_weak_edge(edge);
        }
    }

    pub fn add_weak_edge(&mut self, edge: &MutationGraphEdge) {
        self.weak_edge
            .insert(DirectedEdge::from(&edge), edge.clone());
    }

    pub fn get_node(&self, sha1: &NodeName) -> Option<&MutationGraphNode> {
        self.node.get(sha1)
    }

    pub fn get_edge(&self, arrow: &DirectedEdge) -> Option<&MutationGraphEdge> {
        self.edge.get(arrow)
    }

    pub fn children_of(&self, parent: &NodeName) -> Option<&HashSet<NodeName>> {
        self.children.get(parent)
    }

    pub fn parent_of(&self, child: &NodeName) -> Option<&NodeName> {
        self.parent.get(child)
    }

    pub fn root_of<'a>(&'a self, node: &'a NodeName) -> Result<&'a NodeName> {
        if self.get_node(node).is_none() {
            return Err(MutationGraphError::NodeNotExists(node.clone()));
        }
        match self.parent_of(node) {
            Some(parent) => self.root_of(parent),
            None => Ok(node),
        }
    }

    fn __rank_of(&self, node: &NodeName, rank: usize) -> Result<usize> {
        match self.parent_of(node) {
            Some(parent) => self.__rank_of(parent, rank + 1),
            None => Ok(rank), // If given node is root, then rank is 0.
        }
    }

    pub fn rank_of(&self, node: &NodeName) -> Result<usize> {
        self.__rank_of(node, 0)
    }

    pub fn predecessors_of(&self, node: &NodeName) -> Result<Vec<&NodeName>> {
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

    pub fn self_and_its_predecessors_of(&self, node: &NodeName) -> Result<Vec<&NodeName>> {
        let mut res = self.predecessors_of(node)?;
        match self.get_node(node) {
            Some(node) => res.push(&node.name),
            None => return Err(MutationGraphError::NodeNotExists(node.clone())),
        }
        Ok(res)
    }

    pub fn leaves(&self) -> HashSet<&NodeName> {
        self.children
            .iter()
            .filter(|(_, v)| v.len() == 0)
            .map(|(k, _)| k)
            .collect()
    }

    pub fn roots(&self) -> HashSet<&NodeName> {
        self.node
            .keys()
            .filter(|v| self.parent_of(v).is_none())
            .collect()
    }

    pub fn lookup_by_file_hash(&self, file_hash: &FileHash) -> Result<&NodeName> {
        match self.file_hash.get(file_hash) {
            Some(node_name) => Ok(node_name),
            None => Err(MutationGraphError::FileHashNotExists(file_hash.clone())),
        }
    }

    // Dumps self to dot graph
    pub fn dot_graph(&self, plot_options: PlotOptions) -> Result<String> {
        let predecessors = match plot_options.highlight_edges_from_root_to {
            Some(ref node) => self.predecessors_of(node)?,
            None => vec![],
        };

        let mut res = String::new();

        // Start of dot file
        write!(&mut res, "digraph {{\n").map_err(MutationGraphError::FmtError)?;

        // Add notes
        // NOTE: Add notes first to place notes in preference to edges.
        for (node, label) in plot_options.notate.iter() {
            write!(
                &mut res,
                "{{rank=same; \"note_{node}\" [label=\"{label}\", shape=plaintext, fontname=\"sans-serif\", fontsize=11.0, style=filled, fillcolor=cornsilk];\n\"note_{node}\" -> \"{node}\" [color=black, style=dashed, arrowhead=none, splines=curved]}};\n",
                node=node, label=label
            )
                .map_err(MutationGraphError::FmtError)?;
        }

        // Declare nodes
        let node_heap: BinaryHeap<&MutationGraphNode> = self.node.values().map(|v| v).collect();
        for node in node_heap.into_iter_sorted() {
            let mut additional = String::new();
            if let Some(ref target) = plot_options.highlight_edges_from_root_to {
                if &node.name == target {
                    write!(&mut additional, "color=\"crimson\"")
                        .map_err(MutationGraphError::FmtError)?;
                }
            }
            if plot_options.highlight_crash_input {
                if node.crashed {
                    write!(&mut additional, "shape=\"septagon\", color=\"red4\"")
                        .map_err(MutationGraphError::FmtError)?;
                }
            }
            write!(&mut res, "\"{}\" [{}]\n", node.name, additional)
                .map_err(MutationGraphError::FmtError)?;
        }

        // Declare edges
        let edge_heap: BinaryHeap<&MutationGraphEdge> = self.edge.values().map(|v| v).collect();
        for edge in edge_heap.into_iter_sorted() {
            let mut additional = String::new();
            if let Some(ref target) = plot_options.highlight_edges_from_root_to {
                if predecessors.contains(&&edge.parent)
                    && (predecessors.contains(&&edge.child) || target == &edge.child)
                {
                    write!(&mut additional, ", color=\"crimson\", penwidth=1.21")
                        .map_err(MutationGraphError::FmtError)?;
                }
            } else if plot_options.highlight_edge_with_blue.contains(edge) {
                write!(&mut additional, ", color=\"blue\"")
                    .map_err(MutationGraphError::FmtError)?;
            } else if plot_options.highlight_edge_with_red.contains(edge) {
                write!(&mut additional, ", color=\"red\"").map_err(MutationGraphError::FmtError)?;
            } else if plot_options.highlight_edge_with_green.contains(edge) {
                write!(&mut additional, ", color=\"darkgreen\"")
                    .map_err(MutationGraphError::FmtError)?;
            }

            write!(
                &mut res,
                "\"{}\" -> \"{}\" [label=\"{}\", splines=curved{}];\n",
                edge.parent, edge.child, edge.label, additional
            )
            .map_err(MutationGraphError::FmtError)?;
        }
        for weak_edge in self.weak_edge.values() {
            let mut additional = String::new();
            if plot_options.highlight_edge_with_blue.contains(weak_edge) {
                write!(&mut additional, ", color=\"blue\"")
                    .map_err(MutationGraphError::FmtError)?;
            } else if plot_options.highlight_edge_with_red.contains(weak_edge) {
                write!(&mut additional, ", color=\"red\"").map_err(MutationGraphError::FmtError)?;
            } else if plot_options.highlight_edge_with_green.contains(weak_edge) {
                write!(&mut additional, ", color=\"darkgreen\"")
                    .map_err(MutationGraphError::FmtError)?;
            }

            write!(
                &mut res,
                "\"{}\" -> \"{}\" [label=\"{}\", style=dashed{}];\n",
                weak_edge.parent, weak_edge.child, weak_edge.label, additional
            )
            .map_err(MutationGraphError::FmtError)?;
        }

        // End of dot file
        write!(&mut res, "}}\n").map_err(MutationGraphError::FmtError)?;

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::iter::FromIterator;

    use crate::seed_tree::error::MutationGraphError;
    use crate::seed_tree::mutation_graph_edge::MutationGraphEdge;
    use crate::seed_tree::mutation_graph_node::MutationGraphNode;
    use crate::seed_tree::node_name::NodeName;
    use crate::seed_tree::MutationGraph;

    impl MutationGraphEdge {
        pub fn new(parent: &NodeName, child: &NodeName) -> Self {
            Self {
                parent: parent.clone(),
                child: child.clone(),
                label: String::from(""),
            }
        }
    }

    #[test]
    fn test_mutation_graph_node() {
        let node_1_sha1 = NodeName::from("node_1");
        let no_such_node_sha1 = NodeName::from("no_such_node");

        let mut graph = MutationGraph::new();

        let node_1 = MutationGraphNode::new(&node_1_sha1);
        graph.add_node(&node_1);

        assert_eq!(graph.get_node(&node_1_sha1), Some(&node_1));
        assert_eq!(graph.get_node(&no_such_node_sha1), None);
    }

    #[test]
    fn test_mutation_graph_edge() {
        let node_1_sha1 = NodeName::from("node_1");
        let node_2_sha1 = NodeName::from("node_2");
        let node_3_sha1 = NodeName::from("node_3");
        let node_4_sha1 = NodeName::from("node_4");
        let node_5_sha1 = NodeName::from("node_5");
        let no_such_node_sha1 = NodeName::from("no_such_node");

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

        assert_eq!(graph.rank_of(&node_1_sha1), Ok(0));
        assert_eq!(graph.rank_of(&node_5_sha1), Ok(3));
    }

    #[test]
    fn test_mutation_graph_missing_explicit_node_decl() {
        let node_1_sha1 = NodeName::from("node_1");
        let node_2_sha1 = NodeName::from("node_2");
        let node_3_sha1 = NodeName::from("node_3");

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
                .map(|v| &v.name)
                .collect::<HashSet<&NodeName>>(),
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
        let node_1_sha1 = NodeName::from("node_1");
        let node_2_sha1 = NodeName::from("node_2");
        let node_3_sha1 = NodeName::from("node_3");

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
