use crate::mutation_graph::sha1_string::Sha1String;

#[derive(Debug)]
pub enum PlotOption {
    HighlightEdgesFromRootTo(Sha1String),
}
