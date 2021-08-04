#![feature(binary_heap_into_iter_sorted)]

pub mod mutation_graph;
mod subcommand;

extern crate binary_diff;
extern crate clap;
extern crate log;
extern crate regex;

use clap::{App, Arg, SubCommand};
use std::path::Path;

use crate::mutation_graph::parser::parse_mutation_graph_file;
use crate::subcommand::deriv::deriv;
use crate::subcommand::leaves::leaves;
use crate::subcommand::ls::ls;
use crate::subcommand::origin::origin;
use crate::subcommand::plot::plot;
use crate::subcommand::pred::pred;
use crate::subcommand::roots::roots;

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
                        .help("Output highlighted mutation graph in dot format")
                        .takes_value(false),
                )
        )
        .subcommand(
            SubCommand::with_name("origin")
                .about("Find origin seeds on each offset of SHA1")
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
        origin(matches, graph)
    } else if let Some(matches) = matches.subcommand_matches("plot") {
        plot(matches, graph, mutation_graph_file)
    } else {
        eprintln!("[!] No subcommand specified")
    }
}
