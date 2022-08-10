use crate::seed_tree::mutation_graph_edge::MutationGraphEdge;
use crate::seed_tree::node_name::NodeName;

type Label = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlotOption {
    HighlightEdgesFromRootTo(NodeName),
    HighlightEdgeWithBlue(MutationGraphEdge),
    HighlightEdgeWithRed(MutationGraphEdge),
    HighlightEdgeWithGreen(MutationGraphEdge),
    HighlightCrashInput,
    NotateTo(NodeName, Label),
}
