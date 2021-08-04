use crate::mutation_graph::directed_edge::DirectedEdge;
use crate::mutation_graph::mutation_graph_edge::MutationGraphEdge;
use crate::mutation_graph::plot_options::plot_option::PlotOption;
use crate::mutation_graph::plot_options::PlotOptions;
use crate::mutation_graph::sha1_string::Sha1String;
use crate::mutation_graph::MutationGraph;
use binary_diff::{BinaryDiff, BinaryDiffAnalyzer, BinaryDiffChunk};
use clap::ArgMatches;
use std::io::BufReader;
use std::path::Path;

pub(crate) fn deriv(matches: &ArgMatches, mut graph: MutationGraph) {
    let node = match matches.value_of("SHA1") {
        Some(v) => v.to_string(),
        None => {
            eprintln!("[!] SHA1 is not specified");
            return;
        }
    };
    let offset = match matches.value_of("OFFSET") {
        Some(v) => usize::from_str_radix(v, 16).unwrap(),
        None => {
            eprintln!("[!] OFFSET is not specified");
            return;
        }
    };

    match graph.self_and_its_predecessors_of(&node) {
        Ok(predecessors) => {
            if predecessors.len() > 0 {
                if let Some(seeds_dir_string) = matches.value_of("SEEDS_DIR") {
                    let seeds_dir = Path::new(seeds_dir_string);
                    let seeds: Vec<Sha1String> = predecessors
                        .iter()
                        .filter(|name| seeds_dir.join(&name).exists())
                        .map(|v| Sha1String::from(v.clone()))
                        .collect();

                    if seeds.len() < 2 {
                        eprintln!("[!] None of predecessors of given SHA1 does not exist in given path: SHA1={}, SEEDS_DIR={}", &node, seeds_dir_string);
                    }

                    let mut plot_option: Vec<PlotOption> = vec![];

                    let mut target_offset = offset;
                    for (name_1, name_2) in seeds[0..seeds.len() - 1]
                        .iter()
                        .rev()
                        .zip(seeds[1..seeds.len()].iter().rev())
                    {
                        log::trace!("{} -> {}", name_1, name_2);

                        let diff_chunks = {
                            let file_1 = std::fs::File::open(seeds_dir.join(&name_1)).unwrap();
                            let file_2 = std::fs::File::open(seeds_dir.join(&name_2)).unwrap();

                            BinaryDiff::new(
                                &mut BufReader::new(file_1),
                                &mut BufReader::new(file_2),
                            )
                            .unwrap()
                        };
                        let mut analyze = {
                            let patched_file =
                                std::fs::File::open(seeds_dir.join(&name_2)).unwrap();
                            BinaryDiffAnalyzer::new(&diff_chunks, patched_file)
                        };

                        if matches.is_present("plot") {
                            match analyze.derives_from(target_offset).unwrap() {
                                Some(derives_from) => {
                                    let edge = match graph
                                        .get_edge(&DirectedEdge::new(name_1, name_2))
                                    {
                                        Some(edge) => edge.clone(),
                                        None => {
                                            log::warn!("Edge {} -> {} is not found in graph (potential bug). Added as weak edge", name_1, name_2);
                                            let edge = MutationGraphEdge {
                                                parent: name_1.clone(),
                                                child: name_2.clone(),
                                                label: Sha1String::new(),
                                            };
                                            graph.add_weak_edge(&edge);
                                            edge
                                        }
                                    };
                                    match derives_from.chunk() {
                                        BinaryDiffChunk::Same(_, _) => plot_option
                                            .push(PlotOption::HighlightEdgeWithBlue(edge)),
                                        BinaryDiffChunk::Delete(_, _) => {
                                            plot_option
                                                .push(PlotOption::HighlightEdgeWithBlue(edge));
                                            break;
                                        }
                                        BinaryDiffChunk::Insert(_, _)
                                        | BinaryDiffChunk::Replace(_, _, _) => {
                                            plot_option
                                                .push(PlotOption::HighlightEdgeWithGreen(edge));
                                            break;
                                        }
                                    }
                                }
                                None => break,
                            }
                        } else {
                            match analyze.derives_from(target_offset).unwrap() {
                                Some(derives_from) => {
                                    println!("{} -> {}", name_1, name_2);
                                    match derives_from.original_position() {
                                        Some(original_position) => {
                                            target_offset = original_position;
                                            println!(
                                                "\tat position {:#x} in original file",
                                                original_position
                                            );
                                            println!("\t{}", derives_from.chunk());
                                        }
                                        None => {
                                            println!(
                                                "\tat relative position {:#x} in chunk",
                                                derives_from.relative_position()
                                            );
                                            println!("\t{}", derives_from.chunk());
                                            break;
                                        }
                                    }
                                }
                                None => break,
                            }
                            println!()
                        }
                    }

                    if matches.is_present("plot") {
                        let dot_graph = graph
                            .dot_graph(PlotOptions::from(plot_option.as_slice()).unwrap())
                            .unwrap();
                        print!("{}", dot_graph);
                    }
                } else {
                    eprintln!("[!] SEEDS_DIR is not specified")
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
