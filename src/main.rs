pub mod mutation_graph;

extern crate clap;
use clap::{Arg, App, SubCommand};
extern crate regex;

use std::path::Path;
use crate::mutation_graph::perser::parse_mutation_graph_file;


fn main() {
    let matches = App::new("libfuzzer-mutation-graph-tool")
        .version("1.0")
        .author("Nao Tomori (@K_atc)")
        .about("A Tool to interact wth libfuzzer's mutation graph file")
        .subcommand(SubCommand::with_name("parse")
            .about("Just parse mutation graph file")
        )
        .subcommand(SubCommand::with_name("pred")
            .about("List predecessor of given node")
            .arg(Arg::with_name("SHA1")
                .help("SHA1 (a node name; i.e. seed file name)")
                .required(true)
                .index(1))
        )
        // .subcommand(SubCommand::with_name("succ")
        //     .about("List successors of SHA1")
        //     .arg(Arg::with_name("SHA1")
        //         .help("SHA1 (a node name; i.e. seed file name)")
        //         .required(true)
        //         .index(1))
        // )
        .arg(Arg::with_name("FILE")
            .help("A mutation graph file")
            .required(true)
            .index(1))
        .get_matches();

    let mutation_graph_file = Path::new(matches.value_of("FILE").unwrap());

    let graph = match parse_mutation_graph_file(mutation_graph_file) {
        Ok(graph) => graph,
        Err(why) => {
            eprintln!("[!] Failed to parse file {:?}: {:?}", mutation_graph_file, why);
            return
        }
    };

    if let Some(_) = matches.subcommand_matches("parse") {
        println!("{:#?}", graph);
    } else if let Some(matches) = matches.subcommand_matches("pred") {
        let node = match matches.value_of("SHA1") {
            Some(node) => node.to_string(),
            None => {
                eprintln!("[!] SHA1 is not specified");
                return
            }
        };
        match graph.predecessors_of(&node) {
            Ok(predecessors) => {
                if predecessors.len() > 0 {
                    for name in predecessors.iter() {
                        println!("{}", name)
                    }
                } else {
                    eprintln!("[*] Given node does not have predecessors: sha1={}", node);
                }
            },
            Err(why) => {
                eprintln!("[!] Failed to get predecessors of {}: {:?}", node, why)
            }
        }
    } else {
        eprintln!("[!] No subcommand specified")
    }
}
