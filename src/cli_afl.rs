#![feature(binary_heap_into_iter_sorted)]

pub mod seed_tree;
pub mod subcommand;

extern crate clap;
extern crate log;
extern crate regex;

use crate::seed_tree::parser::afl::{parse_afl_input_directories, AFLExtensions};
use crate::seed_tree::plot_options::PlotOptions;
use crate::subcommand::afl::plot::plot;
use crate::subcommand::common::leaves::leaves;
use crate::subcommand::common::roots::roots;
use crate::subcommand::common::max_rank::max_rank;
use clap::{App, Arg, SubCommand};
use std::path::Path;
use crate::seed_tree::plot_options::plot_option::PlotOption;

fn main() {
    env_logger::init();

    let matches = App::new("seed-tree-analyzer-afl")
        .version("1.0")
        .author("Nao Tomori (@K_atc)")
        .about("A Tool to interact with AFL's seed tree described in inputs file name.")
        .arg(
            Arg::with_name("INPUT_DIR")
                .help("Directories contains AFL's input files.")
                .required(false)
                .index(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name("CRASH_INPUT_DIR")
                .long("crash")
                .help("Enable crash exploration extension. Treat CRASH_INPUT_DIR as a directory contains crash inputs. These are highlighted in the seed tree.")
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
        .subcommand(SubCommand::with_name("maxrank").about("List nodes at maximum rank."))
        .get_matches();

    // NOTE: `&str` is no problem. `parse_afl_input_directories()` converts to Path
    let mut input_dirs: Vec<&str> = match matches.values_of("INPUT_DIR") {
        Some(input_dirs) => input_dirs.collect(),
        None => Vec::new(),
    };
    let crash_inputs_dir = match matches.value_of("CRASH_INPUT_DIR") {
        Some(crash_inputs_dir) => {
            input_dirs.push(crash_inputs_dir);
            Some(Path::new(crash_inputs_dir).to_path_buf())
        }
        None => None,
    };

    if input_dirs.len() == 0 {
        panic!("INPUT_DIR and CRASH_INPUT_DIR is blank. See help")
    }

    log::info!("input_dirs = {:?}", input_dirs);
    let extensions = AFLExtensions {
        aurora: matches.is_present("ENABLE_AURORA"),
        crash_inputs_dir,
    };
    log::info!("Extensions: {:?}", extensions);
    let graph = parse_afl_input_directories(input_dirs, &extensions).unwrap();

    if let Some(_matches) = matches.subcommand_matches("parse") {
        println!("{}", graph.dot_graph(PlotOptions::none()).unwrap());
    } else if let Some(matches) = matches.subcommand_matches("plot") {
        let plot_option = match extensions.crash_inputs_dir {
            Some(_) => vec![PlotOption::HighlightCrashInput],
            None => Vec::new()
        };
        plot(matches, graph, plot_option.as_slice());
    } else if let Some(_matches) = matches.subcommand_matches("roots") {
        roots(graph);
    } else if let Some(_matches) = matches.subcommand_matches("leaves") {
        leaves(graph);
    } else if let Some(_matches) = matches.subcommand_matches("maxrank") {
        max_rank(&graph);
    } else {
        eprintln!("[!] No subcommand specified");
    }
}
