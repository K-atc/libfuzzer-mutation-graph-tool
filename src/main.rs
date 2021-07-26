#![feature(binary_heap_into_iter_sorted)]

pub mod mutation_graph;

extern crate binary_diff;
extern crate clap;
extern crate regex;

use clap::{App, Arg, SubCommand};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::{Write, BufReader};
use std::path::{Path};
use std::process::{Command, Stdio};

use crate::mutation_graph::mutation_graph_node::MutationGraphNode;
use crate::mutation_graph::parser::parse_mutation_graph_file;
use crate::mutation_graph::plot_options::plot_option::PlotOption;
use crate::mutation_graph::plot_options::PlotOptions;
use crate::mutation_graph::sha1_string::Sha1String;
use binary_diff::{BinaryDiff, BinaryDiffChunk};

fn main() {
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
        .get_matches();

    let mutation_graph_file = Path::new(matches.value_of("FILE").unwrap());

    let graph = match parse_mutation_graph_file(mutation_graph_file) {
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
                            let file_1 = std::fs::File::open( seeds_dir.join(&name_1)).unwrap();
                            let file_2 = std::fs::File::open( seeds_dir.join(&name_2)).unwrap();

                            println!("{} -> {}", name_1, name_2);
                            let diff_chunks =
                                BinaryDiff::new(&mut BufReader::new(file_1), &mut BufReader::new(file_2)).unwrap();
                            for chunk in diff_chunks.enhance().chunks() {
                                match chunk {
                                    BinaryDiffChunk::Same(_, _) => (), // Not print
                                    _ => println!("\t{}", chunk)
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
