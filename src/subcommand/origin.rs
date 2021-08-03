use crate::mutation_graph::sha1_string::Sha1String;
use crate::mutation_graph::MutationGraph;
use binary_diff::{BinaryDiff, BinaryDiffAnalyzer, DerivesFrom};
use clap::ArgMatches;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::io::BufReader;
use std::path::Path;

pub(crate) fn origin(matches: &ArgMatches, graph: MutationGraph) {
    let node = match matches.value_of("SHA1") {
        Some(v) => v.to_string(),
        None => {
            eprintln!("[!] SHA1 is not specified");
            return;
        }
    };

    let seeds_dir = if let Some(seeds_dir_string) = matches.value_of("SEEDS_DIR") {
        Path::new(seeds_dir_string)
    } else {
        eprintln!("[!] SEEDS_DIR is not specified");
        return;
    };

    match graph.self_and_its_predecessors_of(&node) {
        Ok(predecessors) => {
            if predecessors.len() > 0 {
                let seeds: Vec<Sha1String> = predecessors
                    .iter()
                    .filter(|name| seeds_dir.join(&name).exists())
                    .map(|v| Sha1String::from(v.clone()))
                    .collect();

                if seeds.len() < 2 {
                    eprintln!("[!] None of predecessors of given SHA1 does not exist in given path: SHA1={}, SEEDS_DIR={}", &node, seeds_dir.display());
                }

                let node_file_size =
                    std::fs::metadata(seeds_dir.join(&node)).unwrap().len() as usize;

                let mut origins: Vec<Origin> = (0..node_file_size)
                    .map(|offset| find_origin(seeds_dir, &seeds, offset))
                    .filter(|v| v.is_some())
                    .map(|v| v.unwrap())
                    .collect();
                origins.sort();
                println!("{:#?}", origins);
            } else {
                eprintln!("[!] Given node does not have predecessors: sha1={}", node);
            }
        }
        Err(why) => {
            eprintln!("[!] Failed to get predecessors of {}: {:?}", node, why)
        }
    }
}

#[derive(Debug, Eq)]
struct Origin {
    offset: usize,
    depth: usize,
    node: Sha1String,
}

impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.depth == other.depth
    }
}

impl PartialOrd for Origin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.depth.cmp(&other.depth) == Equal {
            Some(self.offset.cmp(&other.offset))
        } else {
            Some(self.cmp(other))
        }
    }
}

impl Ord for Origin {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth.cmp(&other.depth)
    }
}

fn find_origin(seeds_dir: &Path, seeds: &Vec<Sha1String>, offset: usize) -> Option<Origin> {
    let mut target_offset = offset;
    for (i, (name_1, name_2)) in seeds[0..seeds.len() - 1]
        .iter()
        .rev()
        .zip(seeds[1..seeds.len()].iter().rev())
        .enumerate()
    {
        log::trace!("{} -> {}", name_1, name_2);

        // TODO: Memorize to reduce redundant calculation
        let diff_chunks = {
            let file_1 = std::fs::File::open(seeds_dir.join(&name_1)).unwrap();
            let file_2 = std::fs::File::open(seeds_dir.join(&name_2)).unwrap();

            BinaryDiff::new(&mut BufReader::new(file_1), &mut BufReader::new(file_2)).unwrap()
        };
        let mut analyze = {
            let patched_file = std::fs::File::open(seeds_dir.join(&name_2)).unwrap();
            BinaryDiffAnalyzer::new(&diff_chunks, patched_file)
        };

        match analyze.derives_from(target_offset).unwrap() {
            Some(derives_from) => match derives_from {
                DerivesFrom {
                    position: Some(position),
                    relative_position: _,
                    chunk: _,
                } => target_offset = position,
                DerivesFrom {
                    position: None,
                    relative_position: _,
                    chunk: _,
                } => {
                    return Some(Origin {
                        offset,
                        depth: i + 1,
                        node: name_1.clone(),
                    })
                }
            },
            None => return None,
        }
    }
    Some(Origin {
        offset: seeds.len(),
        depth: seeds.len(),
        node: seeds[0].clone(),
    })
}
