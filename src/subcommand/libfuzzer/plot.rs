use crate::seed_tree::plot_options::plot_option::PlotOption;
use crate::seed_tree::plot_options::PlotOptions;
use crate::seed_tree::sha1_string::Sha1String;
use crate::seed_tree::MutationGraph;
use crate::subcommand::util::plot_dot_graph::plot_dot_graph;
use clap::ArgMatches;
use std::path::Path;

pub(crate) fn plot(matches: &ArgMatches, graph: MutationGraph, mutation_graph_file: &Path, base_plot_options: &[PlotOption]) {
    let mut plot_options = Vec::new();
    plot_options.extend_from_slice(base_plot_options);
    if let Some(v) = matches.value_of("SHA1") {
        plot_options.push(PlotOption::HighlightEdgesFromRootTo(Sha1String::from(v)))
    };

    let dot_graph_text = graph
        .dot_graph(PlotOptions::from(plot_options.as_slice()).unwrap())
        .expect("Failed to generate dot file");

    plot_dot_graph(&dot_graph_text, "svg", &mutation_graph_file);
    if graph.leaves().len() < 2048 {
        plot_dot_graph(&dot_graph_text, "png", &mutation_graph_file);
    } else {
        log::warn!("This seed tree might be too wide. So omitting plotting to PNG file.");
    }
}
