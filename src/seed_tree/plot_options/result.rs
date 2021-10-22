use crate::seed_tree::plot_options::error::PlotOptionError;

pub type Result<T> = std::result::Result<T, PlotOptionError>;
