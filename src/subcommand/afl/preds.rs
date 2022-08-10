use crate::seed_tree::MutationGraph;
use clap::ArgMatches;

enum PrintOption {
    PrintNodeName,
    PrintFilePath,
    PrintMetadata,
}

pub(crate) fn preds(matches: &ArgMatches, graph: &MutationGraph) {
    let node_name = {
        let id = match matches.value_of("ID") {
            Some(node) => node.to_string(),
            None => {
                eprintln!("[!] ID is not specified");
                return;
            }
        };
        if matches.is_present("hash") {
            graph
                .lookup_by_file_hash(&id)
                .expect("Failed to translate given ID to node name")
                .clone()
        } else {
            id
        }
    };

    let print_option = if matches.is_present("meta") {
        PrintOption::PrintMetadata
    } else if matches.is_present("file") {
        PrintOption::PrintFilePath
    } else {
        PrintOption::PrintNodeName
    };

    match graph.self_and_its_predecessors_of(&node_name) {
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
        Err(why) => eprintln!("[!] Node name={} does not exists: {:?}", node_name, why),
    }
}
