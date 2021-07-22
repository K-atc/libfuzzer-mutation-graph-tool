use std::collections::HashSet;

use plot_option::PlotOption;

pub mod error;
pub mod plot_option;
pub mod result;

use crate::mutation_graph::plot_options::error::PlotOptionError;
use crate::mutation_graph::sha1_string::Sha1String;
use result::Result;

#[derive(Debug, Eq, PartialEq)]
pub struct PlotOptions {
    pub highlight_edges_from_root_to: Option<Sha1String>,
}

impl PlotOptions {
    pub fn from(options: &[PlotOption]) -> Result<Self> {
        Ok(Self {
            highlight_edges_from_root_to: {
                let mut nodes: HashSet<Sha1String> = HashSet::new();
                for option in options.iter() {
                    match option {
                        PlotOption::HighlightEdgesFromRootTo(ref v) => {
                            nodes.insert(v.clone());
                        } // _ => (),
                    }
                }
                match nodes.len() {
                    0..=1 => nodes.iter().last().cloned(),
                    _ => {
                        return Err(PlotOptionError::MultiplePredecessorsNotSupported(
                            nodes.clone(),
                        ))
                    }
                }
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::mutation_graph::plot_options::error::PlotOptionError;
    use crate::mutation_graph::plot_options::plot_option::PlotOption;
    use crate::mutation_graph::plot_options::PlotOptions;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn test_plot_options_null() {
        let options = PlotOptions::from(&[]);
        assert_eq!(
            options,
            Ok(PlotOptions {
                highlight_edges_from_root_to: None
            })
        )
    }

    #[test]
    fn test_plot_options_highlight_predecessors_of_one_node() {
        let sha1_1 = String::from("1");
        let options = PlotOptions::from(&[PlotOption::HighlightEdgesFromRootTo(sha1_1.clone())]);
        assert_eq!(
            options,
            Ok(PlotOptions {
                highlight_edges_from_root_to: Some(sha1_1)
            })
        )
    }

    #[test]
    fn test_plot_options_highlight_predecessors_of_multiple_nodes() {
        let sha1_1 = String::from("1");
        let sha1_2 = String::from("2");
        let options = PlotOptions::from(&[
            PlotOption::HighlightEdgesFromRootTo(sha1_1.clone()),
            PlotOption::HighlightEdgesFromRootTo(sha1_2.clone()),
        ]);
        assert_eq!(
            options,
            Err(PlotOptionError::MultiplePredecessorsNotSupported(
                HashSet::from_iter([sha1_1, sha1_2].iter().cloned())
            ))
        )
    }
}
