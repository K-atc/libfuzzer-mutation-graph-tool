use crate::mutation_graph::sha1_string::Sha1String;
use crate::mutation_graph::MutationGraph;
use binary_diff::{BinaryDiff, BinaryDiffChunk};
use clap::ArgMatches;
use std::io::BufReader;
use std::path::Path;

pub(crate) fn pred(matches: &ArgMatches, graph: MutationGraph) {
    let node = match matches.value_of("SHA1") {
        Some(node) => node.to_string(),
        None => {
            eprintln!("[!] SHA1 is not specified");
            return;
        }
    };

    match graph.self_and_its_predecessors_of(&node) {
        Ok(predecessors) => {
            if predecessors.len() > 0 {
                let seeds_dir = if let Some(seeds_dir) = matches.value_of("SEEDS_DIR_TO_EXISTS") {
                    Some(Path::new(seeds_dir))
                } else {
                    if let Some(seeds_dir) = matches.value_of("SEEDS_DIR_TO_DIFF") {
                        Some(Path::new(seeds_dir))
                    } else {
                        None
                    }
                };
                log::info!("seeds_dir = {:?}", seeds_dir);
                if let Some(seeds_dir) = seeds_dir {
                    let seeds: Vec<Sha1String> = predecessors
                        .iter()
                        .filter(|name| seeds_dir.join(&name).exists())
                        .map(|v| Sha1String::from(v.clone()))
                        .collect();
                    log::info!("seeds = {:?}", seeds);

                    if seeds.len() < 2 {
                        eprintln!("[!] None of predecessors of given SHA1 does not exist in given path: SHA1={}, SEEDS_DIR={}", &node, seeds_dir.display());
                        return;
                    }

                    if matches.value_of("SEEDS_DIR_TO_EXISTS").is_some() {
                        for name in seeds.iter() {
                            println!("{}", name);
                        }
                    }
                    if matches.value_of("SEEDS_DIR_TO_DIFF").is_some() {
                        for (name_1, name_2) in seeds[0..seeds.len() - 1]
                            .iter()
                            .zip(seeds[1..seeds.len()].iter())
                        {
                            let file_1 = std::fs::File::open(seeds_dir.join(&name_1)).unwrap();
                            let file_2 = std::fs::File::open(seeds_dir.join(&name_2)).unwrap();

                            println!("{} -> {}", name_1, name_2);
                            let diff_chunks = BinaryDiff::new(
                                &mut BufReader::new(file_1),
                                &mut BufReader::new(file_2),
                            )
                            .unwrap();
                            for chunk in diff_chunks.enhance().chunks() {
                                match chunk {
                                    BinaryDiffChunk::Same(_, _) => (), // Not print
                                    _ => println!("\t{}", chunk),
                                }
                            }
                            println!()
                        }
                    }
                } else {
                    for name in predecessors.iter() {
                        println!("{}", name);
                    }
                }
            } else {
                eprintln!("[!] Given node does not have predecessors: sha1={}", node);
                return;
            }
        }
        Err(why) => {
            eprintln!("[!] Failed to get predecessors of {}: {:?}", node, why);
            return;
        }
    }
}
