use crate::seed_tree::error::MutationGraphError;
use crate::seed_tree::node_name::NodeName;
use crate::seed_tree::MutationGraph;
use clap::ArgMatches;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

enum PrintOption {
    PrintNodeName,
    PrintFilePath,
    PrintMetadata,
}

pub(crate) fn max_rank(matches: &ArgMatches, graph: &MutationGraph) {
    let print_option = if matches.is_present("meta") {
        PrintOption::PrintMetadata
    } else if matches.is_present("file") {
        PrintOption::PrintFilePath
    } else {
        PrintOption::PrintNodeName
    };
    let max_rank_nodes = match get_max_rank_nodes(graph) {
        Ok(v) => v,
        Err(why) => panic!("Failed to get nodes at max rank: {:?}", why),
    };
    let heap: BinaryHeap<Reverse<&&NodeName>> =
        max_rank_nodes.iter().map(|v| Reverse(v)).collect();
    for name in heap.into_iter_sorted() {
        match print_option {
            PrintOption::PrintNodeName => println!("{}", name.0),
            PrintOption::PrintMetadata => println!("{:?}", graph.get_node(name.0).unwrap()),
            PrintOption::PrintFilePath => {
                println!("{}", graph.get_node(name.0).unwrap().file.display())
            }
        }
    }
}

fn get_max_rank_nodes(graph: &MutationGraph) -> Result<HashSet<&NodeName>, MutationGraphError> {
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
