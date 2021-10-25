use crate::seed_tree::MutationGraph;
use clap::ArgMatches;

enum PrintOption {
    PrintNodeName,
    PrintFilePath,
    PrintMetadata,
}

pub(crate) fn nodes(matches: &ArgMatches, graph: &MutationGraph) {
    let print_option = if matches.is_present("meta") {
        PrintOption::PrintMetadata
    } else if matches.is_present("file") {
        PrintOption::PrintFilePath
    } else {
        PrintOption::PrintNodeName
    };
    for node in graph.nodes() {
        match print_option {
            PrintOption::PrintNodeName => println!("{}", node.sha1),
            PrintOption::PrintMetadata => println!("{:?}", node),
            PrintOption::PrintFilePath => println!("{}", node.file.display()),
        }
    }
}
