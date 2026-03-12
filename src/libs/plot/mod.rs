//! Plotting utilities for terminal-based data visualization.
//!
//! This module provides shared functionality for the `tva plot` subcommands,
//! including axis generation, color management, and rendering helpers.

use anyhow::Result;

// Core modules
pub mod axis;
pub mod binning;
pub mod boxplot;
pub mod chart;
pub mod data;
pub mod heatmap;
pub mod regression;
pub mod render;
pub mod scatter;
pub mod stats;

// Re-exports for convenience
pub use binning::{compute_bins_2d, Bin2d, Bin2dConfig};
pub use binning::{freedman_diaconis_bins, sqrt_bins, sturges_bins};
pub use boxplot::{render_boxplot, BoxPlotConfig, BoxPlotData, BoxStatsRender};
pub use data::{
    build_header, load_bin2d_data, load_box_data, load_numeric_column,
    load_scatter_data, parse_columns, parse_single_column, read_headers, ColumnSpec,
};
pub use heatmap::render_heatmap;
pub use scatter::{
    process_scatter_data, render_scatter_chart, ScatterConfig, ScatterDataset,
};

/// Parse chart dimension with support for:
/// - Absolute values (e.g., "80" for 80 characters)
/// - Ratios relative to terminal size (e.g., "0.8" for 80% of terminal)
/// - Ratios > 1.0 to fill terminal (e.g., "1.0" for 100% of terminal)
pub fn parse_chart_dimension(
    value: Option<&String>,
    term_size: u16,
    default: u16,
) -> Result<u16> {
    match value {
        None => Ok(default),
        Some(v) => {
            if v.contains('.') {
                // Ratio relative to terminal size
                let ratio: f64 = v.parse()?;
                let result = (term_size as f64 * ratio).round() as u16;
                Ok(result.max(10)) // Minimum 10 characters
            } else {
                // Absolute value
                let result: u16 = v.parse()?;
                Ok(result.max(10)) // Minimum 10 characters
            }
        }
    }
}
