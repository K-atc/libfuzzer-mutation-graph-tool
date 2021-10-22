use super::result::Result;
use crate::seed_tree::mutation_graph_edge::MutationGraphEdge;
use crate::seed_tree::mutation_graph_node::MutationGraphNode;
use crate::seed_tree::parser::error::ParseError;
use crate::seed_tree::MutationGraph;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn parse_libfuzzer_mutation_graph_file<T: AsRef<Path>>(file: T) -> Result<MutationGraph> {
    let mut graph = MutationGraph::new();

    {
        if file.as_ref().is_dir() {
            return Err(ParseError::UnexpectedDirectoryPath(
                file.as_ref().to_path_buf(),
            ));
        }

        // Mutation graph file syntax
        let node = Regex::new("^\\s*\"([\\d[:alpha:]]+)\"\\s*$").map_err(ParseError::RegexError)?;
        let edge = Regex::new(
            "^\\s*\"(\\w+)\"\\s*\\->\\s*\"(\\w+)\"\\s*\\[label\\s*=\\s*\"(.*)\"\\]\\s*;\\s*$",
        )
        .map_err(ParseError::RegexError)?;

        // Parse lines of given file along with above syntax
        let mut reader = BufReader::new(File::open(file).map_err(ParseError::IoError)?);
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line).map_err(ParseError::IoError)? == 0 {
                // reached EOF
                break;
            }

            if let Some(m) = node.captures(&line) {
                if m.len() == 2 {
                    match m.get(1) {
                        Some(v) => graph.add_node(&MutationGraphNode::new(&v.as_str().to_string())),
                        None => {
                            return Err(ParseError::SyntaxError(
                                "Missing node value",
                                m[0].to_string(),
                            ))
                        }
                    }
                    continue;
                }
            }
            if let Some(m) = edge.captures(&line) {
                if m.len() == 4 {
                    match (m.get(1), m.get(2), m.get(3)) {
                        (Some(parent), Some(child), Some(label)) => {
                            graph.add_edge(&MutationGraphEdge {
                                parent: parent.as_str().to_string(),
                                child: child.as_str().to_string(),
                                label: label.as_str().to_string(),
                            })
                        }
                        _ => {
                            return Err(ParseError::SyntaxError(
                                "Unexpected edge node",
                                m[0].to_string(),
                            ))
                        }
                    }

                    continue;
                }
            }
            return Err(ParseError::UnknownLine(line.clone()));
        }
    }

    Ok(graph)
}
