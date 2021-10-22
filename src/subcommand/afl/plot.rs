use crate::seed_tree::plot_options::plot_option::PlotOption;
use crate::seed_tree::plot_options::PlotOptions;
use crate::seed_tree::sha1_string::Sha1String;
use crate::seed_tree::MutationGraph;
use crate::subcommand::util::plot_dot_graph::plot_dot_graph;
use clap::ArgMatches;
use std::path::Path;

pub(crate) fn plot(matches: &ArgMatches, graph: MutationGraph) {
    let seed_tree_file = match matches.value_of("DOT_FILE") {
        Some(v) => Path::new(v),
        None => return eprintln!("DOT_FILE is not specified"),
    };
    let plot_options = match matches.value_of("ID") {
        Some(v) => vec![PlotOption::HighlightEdgesFromRootTo(Sha1String::from(v))],
        None => vec![],
    };

    let dot_graph_text = graph
        .dot_graph(PlotOptions::from(plot_options.as_slice()).unwrap())
        .expect("Failed to generate dot file");

    plot_dot_graph(&dot_graph_text, "svg", &seed_tree_file);
    if graph.leaves().len() < 2048 {
        plot_dot_graph(&dot_graph_text, "png", &seed_tree_file);
    } else {
        log::warn!("This seed tree might be too wide. So omitting plotting to PNG file.");
    }
}
