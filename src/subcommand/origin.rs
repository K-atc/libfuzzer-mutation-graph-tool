use crate::mutation_graph::plot_options::plot_option::PlotOption;
use crate::mutation_graph::plot_options::PlotOptions;
use crate::mutation_graph::sha1_string::Sha1String;
use crate::mutation_graph::MutationGraph;
use binary_diff::{BinaryDiff, BinaryDiffAnalyzer, BinaryDiffChunk};
use clap::ArgMatches;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::fmt;
use std::io::BufReader;
use std::path::Path;

pub(crate) fn origin(matches: &ArgMatches, graph: MutationGraph) {
    let node = match matches.value_of("SHA1") {
        Some(v) => Sha1String::from(v),
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

                    plot_options.push(PlotOption::HighlightEdgesFromRootTo(node));

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

#[derive(Debug, Eq)]
struct Origin {
    of_offset: usize,
    depth: usize,
    node: Sha1String,
    position: usize,
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

fn find_origin_of(offset: usize, seeds_dir: &Path, seeds: &Vec<Sha1String>) -> Option<Origin> {
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
    use crate::mutation_graph::parser::parse_mutation_graph_file;
    use crate::mutation_graph::sha1_string::Sha1String;
    use crate::subcommand::origin::find_origin_of;
    use std::path::Path;

    #[test]
    fn test_find_origin_of() {
        let seeds_dir = Path::new("test/sample/seeds/fuzzer-test-suite-openssl-1.0.1f/");
        let graph = parse_mutation_graph_file(Path::new(
            "test/sample/mutation_graph_file/fuzzer-test-suite-openssl-1.0.1f.dot",
        ))
        .unwrap();
        let seeds = graph
            .self_and_its_predecessors_of(&Sha1String::from(
                "c298122410da09836c59484e995c287294c31394",
            ))
            .unwrap()
            .iter()
            .filter(|name| seeds_dir.join(&name).exists())
            .map(|v| Sha1String::from(v.clone()))
            .collect();

        // On far node from target node
        assert_eq!(
            find_origin_of(0x13, seeds_dir, &seeds).unwrap().node,
            Sha1String::from("99878cf124782dc6d21f079bb29e0dba54606bbb")
        );
        assert_eq!(
            find_origin_of(0x13, seeds_dir, &seeds).unwrap().position,
            0x1d
        );
        assert_eq!(
            find_origin_of(0x14, seeds_dir, &seeds).unwrap().position,
            0x1e
        );
        assert_eq!(
            find_origin_of(0x16, seeds_dir, &seeds).unwrap().position,
            0x25
        );
        assert_eq!(
            find_origin_of(0x17, seeds_dir, &seeds).unwrap().position,
            0x26
        );
        assert_eq!(
            find_origin_of(0x1c, seeds_dir, &seeds).unwrap().position,
            0x29
        );

        // On in front of target node
        assert_eq!(
            find_origin_of(0x15, seeds_dir, &seeds).unwrap().node,
            Sha1String::from("76e46ec1efcdcb854486037defc3e777a62524ed")
        );
        assert_eq!(
            find_origin_of(0x15, seeds_dir, &seeds).unwrap().position,
            0x15
        );
        assert_eq!(
            find_origin_of(0x18, seeds_dir, &seeds).unwrap().position,
            0x18
        );
        assert_eq!(
            find_origin_of(0x19, seeds_dir, &seeds).unwrap().position,
            0x19
        );

        // On target node
        assert_eq!(
            find_origin_of(0x1a, seeds_dir, &seeds).unwrap().node,
            Sha1String::from("c298122410da09836c59484e995c287294c31394")
        );
        assert_eq!(
            find_origin_of(0x1a, seeds_dir, &seeds).unwrap().position,
            0x1a
        );
    }
}
