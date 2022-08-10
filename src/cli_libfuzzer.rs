#![feature(binary_heap_into_iter_sorted)]

pub mod seed_tree;
pub mod subcommand;

extern crate base16ct;
extern crate binary_diff;
extern crate clap;
extern crate log;
extern crate regex;
extern crate sha1;

use clap::{App, Arg, SubCommand};
use std::path::Path;
use std::path::PathBuf;

use crate::seed_tree::parser::libfuzzer::parse_libfuzzer_mutation_graph_file;
use crate::subcommand::common::roots::roots;
use crate::subcommand::libfuzzer::deriv::deriv;
use crate::subcommand::libfuzzer::ls::ls;
use crate::subcommand::libfuzzer::origin::origin;
use crate::subcommand::libfuzzer::plot::plot;
use crate::subcommand::libfuzzer::pred::pred;
use subcommand::common::leaves::leaves;

fn main() {
    env_logger::init();

    let matches = App::new("seed-tree-analyzer-libfuzzer")
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
                    Arg::with_name("SEEDS_DIR_TO_DIFF")
                        .long("diff")
                        .help("Diff seeds locate in SEEDS_DIR")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("SEEDS_DIR_TO_EXISTS")
                        .long("exists")
                        .help("List predecessors locate in SEEDS_DIR")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("NODE_NAME")
                        .help("NODE_NAME (a node name; i.e. seed file name)")
                        .required(true)
                        .index(1),
                )
        )
        .subcommand(SubCommand::with_name("plot").about(
            "Plot mutation graph file and save as PNG, SVG.\nThis command requires graphviz.",
        ).arg(
            Arg::with_name("NODE_NAME")
                .help("Highlight edges from root to NODE_NAME")
                .index(1),
        ))
        .subcommand(
            SubCommand::with_name("deriv")
                .about("Analyze derivation of OFFSET of NODE_NAME")
                .arg(
                    Arg::with_name("NODE_NAME")
                        .help("NODE_NAME (a node name; i.e. seed file name)")
                        .required(true)
                        .takes_value(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("OFFSET")
                        .help("Offset of NODE_NAME")
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
                        .help("Output highlighted mutation graph in dot format")
                        .takes_value(false),
                )
        )
        .subcommand(
            SubCommand::with_name("origin")
                .about("Find origin seeds on each offset of NODE_NAME")
                .arg(
                    Arg::with_name("SEEDS_DIR")
                        .help("Seed files location")
                        .required(true)
                        .takes_value(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("NODE_NAME")
                        .help("NODE_NAME (a node name; i.e. seed file name)")
                        .required(true)
                        .takes_value(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("MINIMIZED_CRASH_INPUT")
                        .help("If MINIMIZED_CRASH_INPUT is specified, offsets deleted by MINIMIZED_CRASH_INPUT are ignored during origin analysis")
                        .required(false)
                        .takes_value(true)
                        .index(3),
                )
                .arg(
                    Arg::with_name("plot")
                        .long("plot")
                        .help("Output notated mutation graph in dot format")
                        .takes_value(false),
                )
        )
        .get_matches();

    let mutation_graph_file = match matches.value_of("FILE") {
        Some(path) => Path::new(path),
        None => {
            eprintln!("[!] FILE is not specified");
            return;
        }
    };
    let graph = match parse_libfuzzer_mutation_graph_file(mutation_graph_file) {
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
        ls(graph)
    } else if let Some(_matches) = matches.subcommand_matches("leaves") {
        leaves(graph)
    } else if let Some(_matches) = matches.subcommand_matches("roots") {
        roots(graph)
    } else if let Some(matches) = matches.subcommand_matches("pred") {
        pred(matches, graph)
    } else if let Some(matches) = matches.subcommand_matches("deriv") {
        deriv(matches, graph)
    } else if let Some(matches) = matches.subcommand_matches("origin") {
        let additional_file = match matches.value_of("MINIMIZED_CRASH_INPUT") {
            Some(additional_file) => Some(PathBuf::from(additional_file)),
            None => None,
        };
        origin(matches, graph, additional_file)
    } else if let Some(matches) = matches.subcommand_matches("plot") {
        plot(matches, graph, mutation_graph_file, &[])
    } else {
        eprintln!("[!] No subcommand specified")
    }
}
