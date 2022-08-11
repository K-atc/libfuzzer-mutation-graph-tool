#![feature(binary_heap_into_iter_sorted)]

pub mod seed_tree;
pub mod subcommand;

extern crate base16ct;
extern crate clap;
extern crate log;
extern crate regex;
extern crate sha1;

use crate::seed_tree::parser::afl::{parse_afl_input_directories, AFLExtensions};
use crate::seed_tree::parser::generic::parse_generic_seed_tree_file;
use crate::seed_tree::plot_options::plot_option::PlotOption;
use crate::seed_tree::plot_options::PlotOptions;
use crate::subcommand::afl::filter::filter;
use crate::subcommand::afl::plot::plot;
use crate::subcommand::afl::preds::preds;
use crate::subcommand::common::children::children;
use crate::subcommand::common::leaves::leaves;
use crate::subcommand::common::max_rank::max_rank;
use crate::subcommand::common::nodes::nodes;
use crate::subcommand::common::roots::roots;
use clap::{App, Arg, SubCommand};
use std::collections::HashSet;
use std::path::Path;

fn main() {
    env_logger::init();

    let matches = App::new("seed-tree-analyzer-afl-aurora-crash-exploration")
        .version("1.0")
        .author("Nao Tomori (@K_atc)")
        .about("A Tool to interact with AFL's seed tree described in inputs file name.")
        .arg(
            Arg::with_name("INPUT_DIR")
                .help("Directories contains AFL's input files (NOT crash file).")
                .required(false)
                .index(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name("CRASH_INPUT_DIR")
                .long("crash")
                .help("Treat CRASH_INPUT_DIR as a directory contains crash inputs. These are highlighted in the seed tree. Default is a directory named \"crashes\" treated as crash input directory")
                .takes_value(true),
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
        .subcommand(SubCommand::with_name("leaves").about("List leaf nodes."))
        .subcommand(
            SubCommand::with_name("maxrank")
                .about("List nodes at maximum rank.")
                .arg(
                    Arg::with_name("meta")
                        .long("meta")
                        .takes_value(false)
                        .help("Print metadata of nodes")
                )
                .arg(
                    Arg::with_name("file")
                        .long("file")
                        .takes_value(false)
                        .help("Print file path of nodes. This option cannot be enabled with --meta")
                )
        )
        .subcommand(
            SubCommand::with_name("filter")
                .about("Filter seed tree using commandline options and print it as DOT graph")
                .arg(
                    Arg::with_name("PRED_ID")
                        .long("pred")
                        .takes_value(true)
                        .help("Pick predecessors of PRED_ID"),
                )
                .arg(
                    Arg::with_name("leaves")
                        .long("leaves")
                        .takes_value(false)
                        .help("Pick leaves of picked nodes"),
                )
                .arg(
                    Arg::with_name("meta")
                        .long("meta")
                        .takes_value(false)
                        .help("Print metadata of nodes")
                )
                .arg(
                    Arg::with_name("file")
                        .long("file")
                        .takes_value(false)
                        .help("Print file path of nodes. This option cannot be enabled with --meta")
                )
        )
        .subcommand(
            SubCommand::with_name("children")
                .about("List children of node ID")
                .arg(
                    Arg::with_name("ID")
                        .help("Node ID")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("nodes")
                .about("List nodes at maximum rank.")
                .arg(
                    Arg::with_name("meta")
                        .long("meta")
                        .takes_value(false)
                        .help("Print metadata of nodes")
                )
                .arg(
                    Arg::with_name("file")
                        .long("file")
                        .takes_value(false)
                        .help("Print file path of nodes. This option cannot be enabled with --meta")
                )
        )
        .subcommand(
            SubCommand::with_name("preds")
                .about("List predecessors of given ID and ID. Default of ID is node number")
                .arg(
                    Arg::with_name("ID")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::with_name("meta")
                        .long("meta")
                        .takes_value(false)
                        .help("Print metadata of matched nodes")
                )
                .arg(
                    Arg::with_name("file")
                        .long("file")
                        .takes_value(false)
                        .help("Print file path of matched nodes. This option cannot be enabled with --meta")
                )
                .arg(
                    Arg::with_name("hash")
                        .long("hash")
                        .takes_value(false)
                        .help("ID is meant to be sha1 hash of file")
                )
        )
        .get_matches();

    if matches.subcommand_name().is_none() {
        eprintln!("[!] No subcommand specified. Exit");
        return;
    }

    // NOTE: `&str` is no problem. `parse_afl_input_directories()` converts to Path
    let mut input_dirs: HashSet<&str> = match matches.values_of("INPUT_DIR") {
        Some(input_dirs) => input_dirs.collect(),
        None => HashSet::new(),
    };

    if input_dirs.len() == 0 {
        log::info!("Reading seed tree from stdin")
    }
    log::info!("input_dirs = {:?}", input_dirs);

    let crash_inputs_dir = match matches.value_of("CRASH_INPUT_DIR") {
        Some(crash_inputs_dir) => {
            input_dirs.insert(crash_inputs_dir);
            Some(Path::new(crash_inputs_dir).to_path_buf())
        }
        None => None,
    };

    let extensions = AFLExtensions {
        aurora: matches.is_present("ENABLE_AURORA"),
        crash_inputs_dir,
    };
    log::info!("Extensions: {:?}", extensions);

    let graph = if input_dirs.len() > 0 {
        parse_afl_input_directories(input_dirs, &extensions)
            .expect("Failed to parse input directories")
    } else {
        parse_generic_seed_tree_file(std::io::stdin()).unwrap()
    };

    let base_plot_option = match extensions.crash_inputs_dir {
        Some(_) => vec![PlotOption::HighlightCrashInput],
        None => Vec::new(),
    };

    if let Some(_matches) = matches.subcommand_matches("parse") {
        println!("{}", graph.dot_graph(PlotOptions::none()).unwrap());
    } else if let Some(matches) = matches.subcommand_matches("plot") {
        plot(matches, graph, base_plot_option.as_slice());
    } else if let Some(_matches) = matches.subcommand_matches("roots") {
        roots(graph);
    } else if let Some(_matches) = matches.subcommand_matches("leaves") {
        leaves(graph);
    } else if let Some(matches) = matches.subcommand_matches("maxrank") {
        max_rank(matches, &graph);
    } else if let Some(matches) = matches.subcommand_matches("filter") {
        filter(matches, &graph, base_plot_option.as_slice());
    } else if let Some(matches) = matches.subcommand_matches("children") {
        children(matches, &graph);
    } else if let Some(matches) = matches.subcommand_matches("nodes") {
        nodes(matches, &graph);
    } else if let Some(matches) = matches.subcommand_matches("preds") {
        preds(matches, &graph);
    } else {
        eprintln!("[!] No subcommand specified");
    }
}
