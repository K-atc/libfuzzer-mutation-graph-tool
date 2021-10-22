use crate::seed_tree::mutation_graph_edge::MutationGraphEdge;
use crate::seed_tree::sha1_string::Sha1String;

type Label = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlotOption {
    HighlightEdgesFromRootTo(Sha1String),
    HighlightEdgeWithBlue(MutationGraphEdge),
    HighlightEdgeWithRed(MutationGraphEdge),
    HighlightEdgeWithGreen(MutationGraphEdge),
    HighlightCrashInput,
    NotateTo(Sha1String, Label),
}