use crate::seed_tree::node_name::NodeName;
use crate::seed_tree::plot_options::plot_option::PlotOption;
use crate::seed_tree::plot_options::PlotOptions;
use crate::seed_tree::MutationGraph;
use binary_diff::{BinaryDiff, BinaryDiffAnalyzer, BinaryDiffChunk};
use clap::ArgMatches;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

pub(crate) fn origin(
    matches: &ArgMatches,
    graph: MutationGraph,
    minimized_crash_input: Option<PathBuf>,
) {
    let node = match matches.value_of("NODE_NAME") {
        Some(v) => NodeName::from(v),
        None => {
            eprintln!("[!] NODE_NAME is not specified");
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
            if predecessors.len() > 1 {
                let seeds: Vec<NodeName> = predecessors
                    .iter()
                    .filter(|name| seeds_dir.join(&name).exists())
                    .map(|v| NodeName::from(v.clone()))
                    .collect();

                let node_file_size =
                    std::fs::metadata(seeds_dir.join(&node)).unwrap().len() as usize;

                let ignored_offsets = match minimized_crash_input {
                    Some(ref minimized_crash_input) => calculate_deleted_offsets(
                        BufReader::new(File::open(seeds_dir.join(node.clone())).unwrap()),
                        BufReader::new(File::open(minimized_crash_input).unwrap()),
                    ),
                    None => HashSet::new(),
                };
                if let Some(ref minimized_crash_input) = minimized_crash_input {
                    let mut sorted_ignored_offsets: Vec<&Offset> = ignored_offsets.iter().collect();
                    sorted_ignored_offsets.sort();
                    log::info!(
                        "Offsets {:?} of {} are ignored",
                        sorted_ignored_offsets,
                        minimized_crash_input.display()
                    )
                }

                let mut origins: Vec<Origin> = (0..node_file_size)
                    .filter(|offset| !ignored_offsets.contains(offset))
                    .map(|offset| find_origin_of(offset, seeds_dir, &seeds))
                    .filter(|v| v.is_some())
                    .map(|v| v.unwrap())
                    .collect();

                if matches.is_present("plot") {
                    let mut plot_options: Vec<PlotOption> = origins
                        .iter()
                        .map(|origin| {
                            PlotOption::NotateTo(
                                origin.node.clone(),
                                format!(
                                    "[{:x}] ← {}({:x})",
                                    origin.of_offset,
                                    origin.chunk.name(),
                                    origin.position
                                ),
                            )
                        })
                        .collect();

                    plot_options.push(PlotOption::HighlightEdgesFromRootTo(node.clone()));

                    let dot = graph
                        .dot_graph(PlotOptions::from(plot_options.as_slice()).unwrap())
                        .unwrap();

                    println!("{}", dot);
                } else {
                    origins.sort();
                    for ref origin in origins {
                        println!("{}", origin);
                    }
                }
            } else {
                eprintln!("[!] Given node does not have predecessors: sha1={}", node);
            }
        }
        Err(why) => {
            eprintln!("[!] Failed to get predecessors of {}: {:?}", node, why)
        }
    }
}

type Offset = usize;

#[derive(Debug, Eq)]
struct Origin {
    of_offset: Offset,
    depth: usize,
    node: NodeName,
    position: Offset,
    chunk: BinaryDiffChunk,
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Offset {:#x} derives from {}(position={:#x}) of node \"{}\"",
            self.of_offset,
            self.chunk.name(),
            self.position,
            self.node,
        )
    }
}

impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        self.of_offset == other.of_offset && self.depth == other.depth
    }
}

impl PartialOrd for Origin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.depth.cmp(&other.depth) == Equal {
            Some(self.of_offset.cmp(&other.of_offset))
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

fn calculate_deleted_offsets<R: Read + Seek>(
    mut original: BufReader<R>,
    mut patched: BufReader<R>,
) -> HashSet<Offset> {
    let diff = BinaryDiff::new(&mut original, &mut patched).unwrap();
    log::trace!("diff = {:?}", diff);
    diff.enhance()
        .chunks()
        .iter()
        .map(|v| match v {
            BinaryDiffChunk::Delete(offset, length) => {
                HashSet::from_iter(offset.clone()..offset.clone() + length.clone())
            }
            BinaryDiffChunk::Replace(offset, length, bytes) => {
                if &bytes.len() < length {
                    // // 0    offset
                    // // |-----|<-------->|<----->|
                    // //       |  bytes     range
                    // //       |<---------------->|
                    // //              length
                    // HashSet::from_iter(
                    //     /* range */
                    //     offset.clone() + bytes.len()..offset.clone() + length.clone(),
                    // )
                    HashSet::from_iter(offset.clone()..offset.clone() + length.clone())
                } else {
                    // No offsets deleted (i.e. just updated) by this chunk
                    HashSet::new()
                }
            }
            _ => HashSet::new(),
        })
        .fold(HashSet::new(), |acc, v| acc.union(&v).cloned().collect())
}

fn find_origin_of(offset: usize, seeds_dir: &Path, seeds: &Vec<NodeName>) -> Option<Origin> {
    let mut target_offset = offset;
    for (i, (name_1, name_2)) in seeds[0..seeds.len() - 1]
        .iter()
        .rev()
        .zip(seeds[1..seeds.len()].iter().rev())
        .enumerate()
    {
        log::trace!("{} -> {}", name_1, name_2);

        // TODO: Memorize to reduce redundant calculation
        let diff = {
            let file_1 = std::fs::File::open(seeds_dir.join(&name_1)).unwrap();
            let file_2 = std::fs::File::open(seeds_dir.join(&name_2)).unwrap();

            BinaryDiff::new(&mut BufReader::new(file_1), &mut BufReader::new(file_2)).unwrap()
        };
        let enhanced_diff = diff.enhance();
        let mut analyze = {
            let patched_file = std::fs::File::open(seeds_dir.join(&name_2)).unwrap();
            BinaryDiffAnalyzer::new(&enhanced_diff, patched_file)
        };

        match analyze.derives_from(target_offset).unwrap() {
            Some(derives_from) => match derives_from.original_position() {
                Some(position) => target_offset = position,
                None => {
                    let chunk = derives_from.chunk();
                    return Some(Origin {
                        of_offset: offset,
                        depth: i + 1,
                        node: name_2.clone(), // Derives from this patched binary
                        position: derives_from.patched_position(),
                        chunk: chunk.clone(),
                    });
                }
            },
            None => return None,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::seed_tree::node_name::NodeName;
    use crate::seed_tree::parser::libfuzzer::parse_libfuzzer_mutation_graph_file;
    use crate::subcommand::libfuzzer::origin::{calculate_deleted_offsets, find_origin_of};
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufReader;
    use std::iter::FromIterator;
    use std::path::Path;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_calculate_deleted_offsets() {
        init();

        let crash_input = File::open(Path::new(
            "test/sample/crash-minimization/crash-235641cefe524570bf0df6a3b3722535ce2dbbf7",
        ))
        .unwrap();
        let minimized_crash_input = File::open(Path::new("test/sample/crash-minimization/minimized-from-10dad543216eabe6d97b9d0ba8459215f6dca3f3")).unwrap();
        let result = calculate_deleted_offsets(
            BufReader::new(crash_input),
            BufReader::new(minimized_crash_input),
        );
        // let answer = HashSet::from_iter([1, 2, 3, 8, 9, 13, 14, 15, 16, 18].iter().cloned());
        let answer = HashSet::from_iter([1, 2, 3, 8, 9, 12, 13, 14, 15, 16, 18].iter().cloned());

        log::info!("result - answer = {:?}", result.difference(&answer));
        log::info!("answer - result = {:?}", answer.difference(&result));
        assert_eq!(result, answer);
    }

    #[test]
    fn test_find_origin_of() {
        init();

        let seeds_dir = Path::new("test/sample/seeds/fuzzer-test-suite-openssl-1.0.1f/");
        let graph = parse_libfuzzer_mutation_graph_file(Path::new(
            "test/sample/mutation_graph_file/fuzzer-test-suite-openssl-1.0.1f.dot",
        ))
        .unwrap();
        let seeds = graph
            .self_and_its_predecessors_of(&NodeName::from(
                "c298122410da09836c59484e995c287294c31394",
            ))
            .unwrap()
            .iter()
            .filter(|name| seeds_dir.join(&name).exists())
            .map(|v| NodeName::from(v.clone()))
            .collect();

        // On far node from target node
        assert_eq!(
            find_origin_of(0x14, seeds_dir, &seeds).unwrap().node,
            NodeName::from("99878cf124782dc6d21f079bb29e0dba54606bbb")
        );
        assert_eq!(
            find_origin_of(0x14, seeds_dir, &seeds).unwrap().position,
            0x1e
        );
        assert_eq!(
            find_origin_of(0x16, seeds_dir, &seeds).unwrap().position,
            0x24
        );
        assert_eq!(
            find_origin_of(0x17, seeds_dir, &seeds).unwrap().position,
            0x26
        );
        assert_eq!(
            find_origin_of(0x18, seeds_dir, &seeds).unwrap().position,
            0x29
        );

        // On in front of target node
        assert_eq!(
            find_origin_of(0x15, seeds_dir, &seeds).unwrap().node,
            NodeName::from("76e46ec1efcdcb854486037defc3e777a62524ed")
        );
        assert_eq!(
            find_origin_of(0x15, seeds_dir, &seeds).unwrap().position,
            0x15
        );
        assert_eq!(
            find_origin_of(0x19, seeds_dir, &seeds).unwrap().position,
            0x19
        );
        assert_eq!(
            find_origin_of(0x1b, seeds_dir, &seeds).unwrap().position,
            0x1b
        );
        assert_eq!(
            find_origin_of(0x1c, seeds_dir, &seeds).unwrap().position,
            0x1c
        );

        // On target node
        assert_eq!(
            find_origin_of(0x1a, seeds_dir, &seeds).unwrap().node,
            NodeName::from("c298122410da09836c59484e995c287294c31394")
        );
        assert_eq!(
            find_origin_of(0x1a, seeds_dir, &seeds).unwrap().position,
            0x1a
        );
    }
}
