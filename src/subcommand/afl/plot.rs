use crate::seed_tree::plot_options::plot_option::PlotOption;
use crate::seed_tree::plot_options::PlotOptions;
use crate::seed_tree::node_name::NodeName;
use crate::seed_tree::MutationGraph;
use crate::subcommand::util::plot_dot_graph::plot_dot_graph;
use clap::ArgMatches;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub(crate) fn plot(matches: &ArgMatches, graph: MutationGraph, base_plot_options: &[PlotOption]) {
    let mut plot_options = Vec::new();
    plot_options.extend_from_slice(base_plot_options);
    if let Some(v) = matches.value_of("ID") {
        plot_options.push(PlotOption::HighlightEdgesFromRootTo(NodeName::from(v)))
    };

    let seed_tree_file_name = match matches.value_of("DOT_FILE") {
        Some(v) => Path::new(v),
        None => return eprintln!("DOT_FILE is not specified"),
    };

    let dot_graph_text = graph
        .dot_graph(PlotOptions::from(plot_options.as_slice()).unwrap())
        .expect("Failed to generate dot file");

    File::create(seed_tree_file_name)
    .unwrap()
    .write_all(dot_graph_text.as_bytes())
    .unwrap();
    log::info!(
        "Rendered seed tree to file \"{}\"",
        seed_tree_file_name.display()
    );

    plot_dot_graph(&dot_graph_text, "svg", &seed_tree_file_name);
    if graph.leaves().len() < 2048 {
        plot_dot_graph(&dot_graph_text, "png", &seed_tree_file_name);
    } else {
        log::warn!("This seed tree might be too wide. So omitting plotting to PNG file.");
    }
}
