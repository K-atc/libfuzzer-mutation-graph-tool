#![feature(binary_heap_into_iter_sorted)]

pub mod seed_tree;
pub mod subcommand;

extern crate clap;
extern crate log;
extern crate regex;

use crate::seed_tree::parser::afl::{parse_afl_input_directories, AFLExtensions};
use crate::seed_tree::plot_options::PlotOptions;
use crate::subcommand::afl::plot::plot;
use crate::subcommand::common::roots::roots;
use clap::{App, Arg, SubCommand};

fn main() {
    env_logger::init();

    let matches = App::new("seed-tree-analyzer-afl")
        .version("1.0")
        .author("Nao Tomori (@K_atc)")
        .about("A Tool to interact with AFL's seed tree described in inputs file name.")
        .arg(
            Arg::with_name("INPUT_DIR")
                .help("Directories contains AFL's input files.")
                .required(true)
                .index(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name("ENABLE_AURORA")
                .long("aurora")
                .help("Enable [AUORA] extension. [AUORA] is https://github.com/RUB-SysSec/aurora")
                .takes_value(false),
        )
        .subcommand(
            SubCommand::with_name("parse")
                .about("Scan INPUT_DIR(s) and output seed tree in dot format."),
        )
        .subcommand(
            SubCommand::with_name("plot")
                .about("Plot and save seed tree as DOT, PNG, SVG.\nThis command requires graphviz.")
                .arg(
                    Arg::with_name("DOT_FILE")
                        .help("Path of dot file to be saved")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("ID")
                        .help("Highlight edges from root to ID")
                        .index(2),
                ),
        )
        .subcommand(SubCommand::with_name("roots").about("List root nodes."))
        .get_matches();

    let input_dirs: Vec<&str> = match matches.values_of("INPUT_DIR") {
        Some(input_dirs) => input_dirs.collect(),
        None => panic!("INPUT_DIR is blank"),
    };
    log::info!("input_dirs = {:?}", input_dirs);
    let extensions = AFLExtensions {
        aurora: matches.is_present("ENABLE_AURORA"),
    };
    log::info!(
        "AURORA extension is {}",
        if extensions.aurora {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    let graph = parse_afl_input_directories(input_dirs, &extensions).unwrap();

    if let Some(_matches) = matches.subcommand_matches("parse") {
        println!("{}", graph.dot_graph(PlotOptions::none()).unwrap());
    } else if let Some(matches) = matches.subcommand_matches("plot") {
        plot(matches, graph);
    } else if let Some(_matches) = matches.subcommand_matches("roots") {
        roots(graph);
    } else {
        eprintln!("[!] No subcommand specified");
    }
}
