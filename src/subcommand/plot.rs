use crate::mutation_graph::plot_options::plot_option::PlotOption;
use crate::mutation_graph::plot_options::PlotOptions;
use clap::ArgMatches;
use std::io::Write;
use std::process::{Command, Stdio};
use std::path::Path;
use crate::mutation_graph::sha1_string::Sha1String;
use crate::mutation_graph::MutationGraph;

pub(crate) fn plot(matches: &ArgMatches, graph: MutationGraph, mutation_graph_file: &Path) {
    let plot_options = match matches.value_of("SHA1") {
        Some(v) => vec![PlotOption::HighlightEdgesFromRootTo(Sha1String::from(v))],
        None => vec![],
    };

    let dot_graph_text = graph
        .dot_graph(PlotOptions::from(plot_options.as_slice()).unwrap())
        .expect("Failed to generate dot file");

    plot_dot_graph(&dot_graph_text, "png", &mutation_graph_file);
    plot_dot_graph(&dot_graph_text, "svg", &mutation_graph_file);
}

fn plot_dot_graph(dot_graph_text: &String, format: &'static str, original_file: &Path) {
    let mut child = Command::new("dot")
        .arg(format!("-T{}", format))
        .arg("-o")
        .arg(original_file.with_extension(format).as_os_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn \"dot\" (graphviz)");

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin
            .write_all(dot_graph_text.as_bytes())
            .expect("Failed to write to stdin");
        // Drop `stdin` to close stdin
    }

    let _ = child.wait_with_output().expect("Failed to read stdout");
}
