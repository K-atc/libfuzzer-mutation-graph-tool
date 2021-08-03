use crate::mutation_graph::sha1_string::Sha1String;
use crate::mutation_graph::MutationGraph;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub(crate) fn leaves(graph: MutationGraph) {
    let leaves = graph.leaves();
    let heap: BinaryHeap<Reverse<&&Sha1String>> = leaves.iter().map(|v| Reverse(v)).collect();
    for name in heap.into_iter_sorted() {
        println!("{}", name.0)
    }
}
