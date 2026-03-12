//! Chart rendering utilities for terminal-based plotting.
//!
//! This module provides shared chart configurations and re-exports
//! from specialized rendering modules.

pub use super::scatter::{ScatterConfig, ScatterDataset, render_scatter_chart, process_scatter_data};
pub use super::boxplot::{BoxPlotConfig, render_boxplot, BoxStatsRender};

// Re-export for backward compatibility
pub type ChartConfig = ScatterConfig;
pub type ChartConfigBox = BoxPlotConfig;
pub type PlotDataset = ScatterDataset;
pub type BoxPlotData = super::boxplot::BoxPlotData;

/// Process raw data into datasets ready for rendering.
/// Handles sorting for line mode and prepares regression data.
///
/// # Deprecated
/// Use `process_scatter_data` from `scatter` module instead.
pub fn process_data(
    data: indexmap::IndexMap<String, Vec<(f64, f64)>>,
    is_line: bool,
    regression_data: Vec<(String, Vec<(f64, f64)>, usize)>,
) -> Vec<ScatterDataset> {
    process_scatter_data(data, is_line, regression_data)
}

/// Render a chart to stdout.
///
/// # Deprecated
/// Use `render_scatter_chart` from `scatter` module instead.
pub fn render_chart(
    datasets: Vec<ScatterDataset>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    config: &ScatterConfig,
) -> anyhow::Result<()> {
    render_scatter_chart(datasets, x_min, x_max, y_min, y_max, config)
}

/// Render a box plot to stdout.
///
/// # Deprecated
/// Use `render_boxplot` from `boxplot` module instead.
pub fn render_chart_box<T: BoxStatsRender>(
    box_data: indexmap::IndexMap<String, T>,
    y_min: f64,
    y_max: f64,
    config: &BoxPlotConfig,
) -> anyhow::Result<()> {
    render_boxplot(box_data, y_min, y_max, config)
}
