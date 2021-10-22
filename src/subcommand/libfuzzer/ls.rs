use crate::seed_tree::mutation_graph_node::MutationGraphNode;
use crate::seed_tree::MutationGraph;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub(crate) fn ls(graph: MutationGraph) {
    let heap: BinaryHeap<Reverse<&MutationGraphNode>> = graph.nodes().map(|v| Reverse(v)).collect();
    for node in heap.into_iter_sorted() {
        println!("{}", node.0.sha1)
    }
}
