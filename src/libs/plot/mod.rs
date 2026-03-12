//! Plotting utilities for terminal-based data visualization.
//!
//! This module provides shared functionality for the `tva plot` subcommands,
//! including axis generation, color management, and rendering helpers.

use anyhow::Result;

pub mod axis;
pub mod bin2d;
pub mod chart;
pub mod regression;
pub mod render;
pub mod stats;

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
