use super::error::ParseError;
use super::result::Result;
use crate::seed_tree::mutation_graph_edge::MutationGraphEdge;
use crate::seed_tree::mutation_graph_node::MutationGraphNode;
use crate::seed_tree::MutationGraph;
use regex::Regex;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AFLExtensions {
    pub(crate) aurora: bool,
    pub(crate) crash_inputs_dir: Option<PathBuf>,
}

impl AFLExtensions {
    pub fn none(&self) -> bool {
        !self.aurora
    }

    pub fn aurora(&self) -> bool {
        self.aurora
    }
}

pub fn parse_afl_input_directories<T: AsRef<Path>>(
    directories: Vec<T>,
    extensions: &AFLExtensions,
) -> Result<MutationGraph> {
    let mut res = MutationGraph::new();
    for directory in directories {
        parse_afl_input_directory(directory, &mut res, extensions)?;
    }
    Ok(res)
}

fn parse_afl_input_directory<T: AsRef<Path>>(
    directory: T,
    graph: &mut MutationGraph,
    extensions: &AFLExtensions,
) -> Result<()> {
    if directory.as_ref().is_file() {
        return Err(ParseError::UnexpectedFilePath(
            directory.as_ref().to_path_buf(),
        ));
    }

    visit_directory(directory.as_ref().to_path_buf(), graph, extensions)?;

    Ok(())
}

fn visit_directory(
    directory: PathBuf,
    graph: &mut MutationGraph,
    extensions: &AFLExtensions,
) -> Result<()> {
    log::trace!("Scanning directory {:?}", directory);

    let one_line_info = Regex::new(if extensions.aurora() {
        "^id:(\\S+),(src|orig):([^:]+)(,op:([^_]+)(_(\\S+))?)?$"
    } else {
        "^id:(\\S+),(src|orig):([^:]+)(,op:(\\S+))?$"
    })
    .map_err(ParseError::RegexError)?;

    for entry in directory.read_dir().map_err(ParseError::IoError)? {
        let path = entry.map_err(ParseError::IoError)?.path();
        let file_name = match path.file_name() {
            Some(file_name) => file_name.to_str().ok_or(ParseError::StringEncoding)?,
            // Recursively iterate directory
            None => return visit_directory(path, graph, extensions),
        };
        // log::trace!("parsing file name: {}", file_name);

        let is_crash_input_node = match extensions.crash_inputs_dir {
            Some(ref crash_input_dir) => path.starts_with(crash_input_dir),
            None => false
        };

        match one_line_info.captures(file_name) {
            Some(captures) => {
                let id = match captures.get(1) {
                    Some(id) => {
                        if extensions.aurora() {
                            match captures.get(7) {
                                Some(non_crash_id) => {
                                    format!("{}/{}", id.as_str(), non_crash_id.as_str())
                                }
                                None => id.as_str().to_string(),
                            }
                        } else {
                            id.as_str().to_string()
                        }
                    }
                    None => {
                        return Err(ParseError::SyntaxError(
                            "'id' does not exists",
                            file_name.to_string(),
                        ))
                    }
                };
                graph.add_node(&MutationGraphNode::new_with_metadata(&id.to_string(), is_crash_input_node));

                let src_list = match captures.get(3) {
                    Some(src_list) => src_list.as_str().split("+"),
                    None => {
                        return Err(ParseError::SyntaxError(
                            "'src' does not exists",
                            file_name.to_string(),
                        ))
                    }
                };
                let op = match captures.get(5) {
                    Some(op) => op.as_str(),
                    None => "origin",
                };

                for src in src_list {
                    graph.add_edge(&MutationGraphEdge {
                        parent: src.to_string(),
                        child: id.to_string(),
                        label: op.to_string(),
                    });
                    break; // Ignore splice source input
                }
            }
            None => {
                log::warn!(
                    "file \"{}\" does not have AFL's input file name format",
                    file_name
                )
            }
        }
    }

    Ok(())
}
