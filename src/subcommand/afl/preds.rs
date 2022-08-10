use crate::seed_tree::MutationGraph;
use clap::ArgMatches;

enum PrintOption {
    PrintNodeName,
    PrintFilePath,
    PrintMetadata,
}

pub(crate) fn preds(matches: &ArgMatches, graph: &MutationGraph) {
    let node = match matches.value_of("ID") {
        Some(node) => node.to_string(),
        None => {
            eprintln!("[!] ID is not specified");
            return;
        }
    };

    let print_option = if matches.is_present("meta") {
        PrintOption::PrintMetadata
    } else if matches.is_present("file") {
        PrintOption::PrintFilePath
    } else {
        PrintOption::PrintNodeName
    };

    match graph.self_and_its_predecessors_of(&node) {
        Ok(nodes) => {
            for node in nodes {
                let node = graph.get_node(node).unwrap();
                match print_option {
                    PrintOption::PrintNodeName => println!("{}", node.name),
                    PrintOption::PrintMetadata => println!("{:?}", node),
                    PrintOption::PrintFilePath => println!("{}", node.file.display()),
                }
            }
        }
        Err(why) => eprintln!("[!] Node {} does not exists: {:?}", node, why),
    }
}
