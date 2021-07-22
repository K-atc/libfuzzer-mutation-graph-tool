pub mod mutation_graph;

extern crate clap;
use clap::{Arg, App};
extern crate regex;

use std::path::Path;
use crate::mutation_graph::perser::parse_mutation_graph_file;


fn main() {
    let matches = App::new("My Super Program")
        .version("1.0")
        .author("Nao Tomori (@K_atc)")
        .about("A Tool to interact wth libfuzzer's mutation graph file")
        .arg(Arg::with_name("FILE")
            .help("A mutation graph file")
            .required(true)
            .index(1))
        .get_matches();

    let mutation_graph_file = Path::new(matches.value_of("FILE").unwrap());

    let graph = match parse_mutation_graph_file(mutation_graph_file) {
        Ok(graph) => graph,
        Err(why) => {
            println!("[!] Failed to parse file {:?}: {:?}", mutation_graph_file, why);
            return
        }
    };

    println!("[*] graph = {:#?}", graph);
}
