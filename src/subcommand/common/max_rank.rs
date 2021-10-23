use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use crate::seed_tree::error::MutationGraphError;
use crate::seed_tree::MutationGraph;
use crate::seed_tree::sha1_string::Sha1String;

pub(crate) fn max_rank(graph: &MutationGraph) {
    let max_rank_nodes = match get_max_rank_nodes(graph) {
        Ok(v) => v,
        Err(why) => panic!("Failed to get nodes at max rank: {:?}", why)
    };
    let heap: BinaryHeap<Reverse<&&Sha1String>> = max_rank_nodes.iter().map(|v| Reverse(v)).collect();
    for name in heap.into_iter_sorted() {
        println!("{}", name.0)
    }
}

fn get_max_rank_nodes(graph: &MutationGraph) -> Result<HashSet<&Sha1String>, MutationGraphError> {
    let mut max_rank = 0;
    let mut max_rank_nodes = HashSet::new();
    for leaf in graph.leaves() {
        let rank = graph.rank_of(leaf)?;
        if rank > max_rank {
            max_rank_nodes.clear();
            max_rank = rank;
        }
        if rank == max_rank {
            max_rank_nodes.insert(leaf);
        }
    }
    Ok(max_rank_nodes)
}