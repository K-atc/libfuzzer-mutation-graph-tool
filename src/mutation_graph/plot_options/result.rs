use crate::mutation_graph::plot_options::error::PlotOptionError;

pub type Result<T> = std::result::Result<T, PlotOptionError>;
