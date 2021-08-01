use crate::mutation_graph::sha1_string::Sha1String;
use crate::mutation_graph::mutation_graph_edge::MutationGraphEdge;

#[derive(Debug)]
pub enum PlotOption {
    HighlightEdgesFromRootTo(Sha1String),
    HighlightEdgeWithBlue(MutationGraphEdge),
    HighlightEdgeWithRed(MutationGraphEdge),
    HighlightEdgeWithGreen(MutationGraphEdge),
}
