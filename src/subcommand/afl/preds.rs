use crate::seed_tree::MutationGraph;
use clap::ArgMatches;
use log::info;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[allow(unused)]
enum PrintOption {
    PrintNodeName,
    PrintFilePath,
    PrintMetadata,
}

#[allow(unused)]
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

    let export_dir = matches.value_of("export");
    let mut export_original_file = match export_dir {
        Some(export_dir) => {
            if !PathBuf::from(export_dir).exists() {
                info!("Created export directory");
                fs::create_dir(export_dir);
            }
            let export_original_file = PathBuf::from(export_dir).join("original_file.txt");
            Some(
                File::create(&export_original_file)
                    .expect(format!("Failed to create file: {:?}", &export_original_file).as_str()),
            )
        }
        None => None,
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
                if !node.file.as_os_str().is_empty() {
                    if let Some(ref export_dir) = export_dir {
                        let copy_to = PathBuf::from(export_dir).join(&node.name);
                        fs::copy(/* from */ &node.file, /* to */ &copy_to).expect(
                            format!(
                                "Failed to copy file: from={:?}, to={:?}",
                                node.file, copy_to
                            )
                            .as_str(),
                        );
                        if let Some(ref mut export_original_file) = export_original_file {
                            write!(
                                export_original_file,
                                "{} {}\n",
                                node.hash,
                                node.file.display()
                            )
                            .expect("Failed to write");
                        }
                    }
                }
            }
        }
        Err(why) => eprintln!("[!] Node name={} does not exists: {:?}", node_name, why),
    }
}
