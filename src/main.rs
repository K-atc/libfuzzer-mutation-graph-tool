#![feature(binary_heap_into_iter_sorted)]

pub mod mutation_graph;
mod subcommand;

extern crate binary_diff;
extern crate clap;
extern crate log;
extern crate regex;

use clap::{App, Arg, SubCommand};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::{BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::mutation_graph::directed_edge::DirectedEdge;
use crate::mutation_graph::mutation_graph_edge::MutationGraphEdge;
use crate::mutation_graph::mutation_graph_node::MutationGraphNode;
use crate::mutation_graph::parser::parse_mutation_graph_file;
use crate::mutation_graph::plot_options::plot_option::PlotOption;
use crate::mutation_graph::plot_options::PlotOptions;
use crate::mutation_graph::sha1_string::Sha1String;
use crate::subcommand::origin::origin;
use binary_diff::{BinaryDiff, BinaryDiffAnalyzer, BinaryDiffChunk, DerivesFrom};

fn main() {
    env_logger::init();

    let matches = App::new("libfuzzer-mutation-graph-tool")
        .version("1.0")
        .author("Nao Tomori (@K_atc)")
        .about("A Tool to interact with libfuzzer's mutation graph file.")
        .arg(
            Arg::with_name("FILE")
                .help("A mutation graph file.")
                .required(true)
                .index(1),
        )
        .subcommand(SubCommand::with_name("parse").about("Just parse mutation graph file."))
        .subcommand(SubCommand::with_name("ls").about("List nodes."))
        .subcommand(SubCommand::with_name("leaves").about("List leaf nodes."))
        .subcommand(SubCommand::with_name("roots").about("List root nodes."))
        .subcommand(
            SubCommand::with_name("pred")
                .about("List predecessor of given node.")
                .arg(
                    Arg::with_name("SEEDS_DIR")
                        .long("diff")
                        .help("Diff seeds locate in SEEDS_DIR")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("SHA1")
                        .help("SHA1 (a node name; i.e. seed file name)")
                        .required(true)
                        .index(1),
                )
        )
        .subcommand(SubCommand::with_name("plot").about(
            "Plot mutation graph file and save as PNG, SVG.\nThis command requires graphviz.",
        ).arg(
            Arg::with_name("SHA1")
                .help("Highlight edges from root to SHA1")
                .index(1),
        ))
        .subcommand(
            SubCommand::with_name("deriv")
                .about("Analyze derivation of OFFSET of SHA1")
                .arg(
                    Arg::with_name("SHA1")
                        .help("SHA1 (a node name; i.e. seed file name)")
                        .required(true)
                        .takes_value(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("OFFSET")
                        .help("Offset of SHA1")
                        .required(true)
                        .takes_value(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("SEEDS_DIR")
                        .help("Seed files location")
                        .required(true)
                        .takes_value(true)
                        .index(3),
                )
                .arg(
                    Arg::with_name("plot")
                        .long("plot")
                        .help("Output highlighted mutation graph")
                        .takes_value(false),
                )
        )
        .subcommand(
            SubCommand::with_name("origin")
                .about("Find origin seed of each offset of SHA1")
                .arg(
                    Arg::with_name("SEEDS_DIR")
                        .help("Seed files location")
                        .required(true)
                        .takes_value(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("SHA1")
                        .help("SHA1 (a node name; i.e. seed file name)")
                        .required(true)
                        .takes_value(true)
                        .index(2),
                )
        )
        .get_matches();

    let mutation_graph_file = Path::new(matches.value_of("FILE").unwrap());

    let mut graph = match parse_mutation_graph_file(mutation_graph_file) {
        Ok(graph) => graph,
        Err(why) => {
            eprintln!(
                "[!] Failed to parse file {:?}: {:?}",
                mutation_graph_file, why
            );
            return;
        }
    };

    if let Some(_matches) = matches.subcommand_matches("parse") {
        println!("{:#?}", graph);
    } else if let Some(_matches) = matches.subcommand_matches("ls") {
        let heap: BinaryHeap<Reverse<&MutationGraphNode>> =
            graph.nodes().map(|v| Reverse(v)).collect();
        for node in heap.into_iter_sorted() {
            println!("{}", node.0.sha1)
        }
    } else if let Some(_matches) = matches.subcommand_matches("leaves") {
        let leaves = graph.leaves();
        let heap: BinaryHeap<Reverse<&&Sha1String>> = leaves.iter().map(|v| Reverse(v)).collect();
        for name in heap.into_iter_sorted() {
            println!("{}", name.0)
        }
    } else if let Some(_matches) = matches.subcommand_matches("roots") {
        let leaves = graph.roots();
        let heap: BinaryHeap<Reverse<&&Sha1String>> = leaves.iter().map(|v| Reverse(v)).collect();
        for name in heap.into_iter_sorted() {
            println!("{}", name.0)
        }
    } else if let Some(matches) = matches.subcommand_matches("pred") {
        let node = match matches.value_of("SHA1") {
            Some(node) => node.to_string(),
            None => {
                eprintln!("[!] SHA1 is not specified");
                return;
            }
        };

        match graph.predecessors_of(&node) {
            Ok(predecessors) => {
                if predecessors.len() > 0 {
                    if let Some(seeds_dir_string) = matches.value_of("SEEDS_DIR") {
                        let seeds_dir = Path::new(seeds_dir_string);
                        let mut seeds: Vec<Sha1String> = predecessors
                            .iter()
                            .filter(|name| seeds_dir.join(&name).exists())
                            .map(|v| Sha1String::from(v.clone()))
                            .collect();
                        seeds.push(node.clone());

                        if seeds.len() < 2 {
                            eprintln!("[!] None of predecessors of given SHA1 does not exist in given path: SHA1={}, SEEDS_DIR={}", &node, seeds_dir_string);
                        }

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
                    } else {
                        for name in predecessors.iter() {
                            println!("{}", name);
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
    } else if let Some(matches) = matches.subcommand_matches("deriv") {
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
                                        match derives_from {
                                            DerivesFrom {
                                                position: Some(_),
                                                relative_position: _,
                                                chunk: _,
                                            } => plot_option
                                                .push(PlotOption::HighlightEdgeWithBlue(edge)),
                                            DerivesFrom {
                                                position: None,
                                                relative_position: _,
                                                chunk,
                                            } => {
                                                match chunk {
                                                    BinaryDiffChunk::Delete(_, _) => plot_option
                                                        .push(PlotOption::HighlightEdgeWithBlue(
                                                            edge,
                                                        )),
                                                    BinaryDiffChunk::Insert(_, _) => plot_option
                                                        .push(PlotOption::HighlightEdgeWithGreen(
                                                            edge,
                                                        )),
                                                    _ => log::warn!("Unexpected chunk {:?}", chunk),
                                                }
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
                                        match derives_from {
                                            DerivesFrom {
                                                position: Some(position),
                                                relative_position: _,
                                                chunk,
                                            } => {
                                                target_offset = position;
                                                println!(
                                                    "\tat position {:#x} in original file",
                                                    position
                                                );
                                                println!("\t{}", chunk);
                                            }
                                            DerivesFrom {
                                                position: None,
                                                relative_position,
                                                chunk,
                                            } => {
                                                println!(
                                                    "\tat relative position {:#x} in chunk",
                                                    relative_position
                                                );
                                                println!("\t{}", chunk);
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
    } else if let Some(matches) = matches.subcommand_matches("origin") {
        origin(matches, graph)
    } else if let Some(matches) = matches.subcommand_matches("plot") {
        let plot_options = match matches.value_of("SHA1") {
            Some(v) => vec![PlotOption::HighlightEdgesFromRootTo(Sha1String::from(v))],
            None => vec![],
        };

        let dot_graph_text = graph
            .dot_graph(PlotOptions::from(plot_options.as_slice()).unwrap())
            .expect("Failed to generate dot file");

        plot_dot_graph(&dot_graph_text, "png", &mutation_graph_file);
        plot_dot_graph(&dot_graph_text, "svg", &mutation_graph_file);
    } else {
        eprintln!("[!] No subcommand specified")
    }
}

fn plot_dot_graph(dot_graph_text: &String, format: &'static str, original_file: &Path) {
    let mut child = Command::new("dot")
        .arg(format!("-T{}", format))
        .arg("-o")
        .arg(original_file.with_extension(format).as_os_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn \"dot\" (graphviz)");

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin
            .write_all(dot_graph_text.as_bytes())
            .expect("Failed to write to stdin");
        // Drop `stdin` to close stdin
    }

    let _ = child.wait_with_output().expect("Failed to read stdout");
}
