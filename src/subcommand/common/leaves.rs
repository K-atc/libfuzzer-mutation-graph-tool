use crate::seed_tree::node_name::NodeName;
use crate::seed_tree::MutationGraph;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub(crate) fn leaves(graph: MutationGraph) {
    let leaves = graph.leaves();
    let heap: BinaryHeap<Reverse<&&NodeName>> = leaves.iter().map(|v| Reverse(v)).collect();
    for name in heap.into_iter_sorted() {
        println!("{}", name.0)
    }
}
