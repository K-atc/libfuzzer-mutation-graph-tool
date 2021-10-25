use crate::seed_tree::sha1_string::Sha1String;
use crate::seed_tree::MutationGraph;
use clap::ArgMatches;

pub(crate) fn children(matches: &ArgMatches, graph: &MutationGraph) {
    let node = match matches.value_of("ID") {
        Some(node) => Sha1String::from(node),
        None => panic!("ID is not specified"),
    };
    match graph.children_of(&node) {
        Some(children) => {
            for child in children.iter() {
                println!("{}", child);
            }
        }
        None => eprintln!("[!] Node \"{}\" does not exists", node),
    }
}
