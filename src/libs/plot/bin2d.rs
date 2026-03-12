//! 2D binning and heatmap rendering for terminal-based visualization.
//!
//! This module implements 2D binning (similar to ggplot2's geom_bin2d) for
//! visualizing density distributions of two-dimensional data.

use anyhow::Result;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};
use ratatui::Terminal;

use super::axis;
use super::render;

/// Configuration for 2D binning heatmap rendering.
pub struct Bin2dConfig {
    pub width: u16,
    pub height: u16,
    pub x_bins: usize,
    pub y_bins: usize,
    pub x_binwidth: Option<f64>,
    pub y_binwidth: Option<f64>,
    pub x_label: String,
    pub y_label: String,
}

/// A 2D bin with count information.
#[derive(Debug)]
struct Bin2d {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    count: usize,
}

/// Compute 2D bins from data points.
fn compute_bins_2d(
    x_values: &[f64],
    y_values: &[f64],
    x_bins: usize,
    y_bins: usize,
    x_binwidth: Option<f64>,
    y_binwidth: Option<f64>,
) -> (Vec<Bin2d>, f64, f64, f64, f64) {
    // Compute data bounds
    let x_min = x_values.iter().copied().fold(f64::INFINITY, f64::min);
    let x_max = x_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_min = y_values.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max = y_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // Compute actual bin counts and widths
    let (actual_x_bins, actual_x_binwidth) = match x_binwidth {
        Some(width) => {
            let bins = ((x_max - x_min) / width).ceil() as usize;
            (bins.max(1), width)
        }
        None => {
            let width = (x_max - x_min) / x_bins as f64;
            (x_bins, width)
        }
    };

    let (actual_y_bins, actual_y_binwidth) = match y_binwidth {
        Some(width) => {
            let bins = ((y_max - y_min) / width).ceil() as usize;
            (bins.max(1), width)
        }
        None => {
            let width = (y_max - y_min) / y_bins as f64;
            (y_bins, width)
        }
    };

    // Initialize bins
    let mut bins: Vec<Vec<usize>> = vec![vec![0; actual_y_bins]; actual_x_bins];

    // Count points in each bin
    for (x, y) in x_values.iter().zip(y_values.iter()) {
        let x_bin = ((x - x_min) / actual_x_binwidth)
            .floor()
            .clamp(0.0, (actual_x_bins - 1) as f64) as usize;
        let y_bin = ((y - y_min) / actual_y_binwidth)
            .floor()
            .clamp(0.0, (actual_y_bins - 1) as f64) as usize;
        bins[x_bin][y_bin] += 1;
    }

    // Convert to Bin2d structures
    let mut bin2d_list = Vec::new();
    for i in 0..actual_x_bins {
        for j in 0..actual_y_bins {
            bin2d_list.push(Bin2d {
                x_min: x_min + i as f64 * actual_x_binwidth,
                x_max: x_min + (i + 1) as f64 * actual_x_binwidth,
                y_min: y_min + j as f64 * actual_y_binwidth,
                y_max: y_min + (j + 1) as f64 * actual_y_binwidth,
                count: bins[i][j],
            });
        }
    }

    (bin2d_list, x_min, x_max, y_min, y_max)
}

/// Render density characters based on normalized count.
/// 7-level non-linear scale: low values have finer granularity
fn density_char(density: f64) -> &'static str {
    match density {
        d if d >= 0.8 => "█", // Highest density
        d if d >= 0.6 => "▓",
        d if d >= 0.4 => "▒",
        d if d >= 0.2 => "░",
        d if d >= 0.05 => "·", // Low density threshold
        _ => " ",              // Below 5%: not shown
    }
}

/// Render a 2D binning heatmap to stdout.
pub fn render_bin2d_chart(
    x_values: &[f64],
    y_values: &[f64],
    config: &Bin2dConfig,
) -> Result<()> {
    // Compute bins
    let (bins, x_min, x_max, y_min, y_max) = compute_bins_2d(
        x_values,
        y_values,
        config.x_bins,
        config.y_bins,
        config.x_binwidth,
        config.y_binwidth,
    );

    if bins.is_empty() {
        return Err(anyhow::anyhow!("No bins computed"));
    }

    // Find max count for normalization
    let max_count = bins.iter().map(|b| b.count).max().unwrap_or(1);

    // Generate axis labels
    let x_labels_vec =
        axis::generate_axis_labels(x_min, x_max, config.width as usize, 10, 2, 4);
    let y_labels_vec =
        axis::generate_axis_labels(y_min, y_max, config.height as usize, 4, 2, 4);

    let x_bounds_aligned = if x_labels_vec.len() >= 2 {
        let first: f64 = x_labels_vec.first().unwrap().parse().unwrap_or(x_min);
        let last: f64 = x_labels_vec.last().unwrap().parse().unwrap_or(x_max);
        [first, last]
    } else {
        [x_min, x_max]
    };
    let y_bounds_aligned = if y_labels_vec.len() >= 2 {
        let first: f64 = y_labels_vec.first().unwrap().parse().unwrap_or(y_min);
        let last: f64 = y_labels_vec.last().unwrap().parse().unwrap_or(y_max);
        [first, last]
    } else {
        [y_min, y_max]
    };

    let x_labels: Vec<Span> = x_labels_vec.iter().cloned().map(Span::from).collect();
    let y_labels: Vec<Span> = y_labels_vec.iter().cloned().map(Span::from).collect();

    // Create an empty chart with axes (no data)
    let empty_dataset: Dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Scatter)
        .data(&[]);

    let chart = Chart::new(vec![empty_dataset])
        .x_axis(
            Axis::default()
                .title(Span::from(config.x_label.clone()))
                .style(Style::default().fg(Color::Gray))
                .bounds(x_bounds_aligned)
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title(Span::from(config.y_label.clone()))
                .style(Style::default().fg(Color::Gray))
                .bounds(y_bounds_aligned)
                .labels(y_labels),
        );

    // Render chart to buffer
    let backend = TestBackend::new(config.width, config.height);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let area = Rect::new(0, 0, config.width, config.height);
        f.render_widget(chart, area);

        // Get the inner chart area (where data would be plotted)
        // Chart widget typically has margins for labels
        // We need to estimate the plotting area
        let y_label_width =
            y_labels_vec.iter().map(|l| l.len()).max().unwrap_or(1) as u16;
        let chart_inner_x = y_label_width + 1; // Y labels + axis line
        let _chart_inner_y = 0; // Top margin
        let chart_inner_width = config.width.saturating_sub(chart_inner_x + 1); // -1 for right margin
        let chart_inner_height = config.height.saturating_sub(2); // -2 for X axis and labels

        // Map data coordinates to screen coordinates within chart area
        let x_to_col = |x: f64| -> u16 {
            let ratio =
                (x - x_bounds_aligned[0]) / (x_bounds_aligned[1] - x_bounds_aligned[0]);
            let col = chart_inner_x + (ratio * (chart_inner_width - 1) as f64) as u16;
            col.clamp(chart_inner_x, config.width - 1)
        };

        let y_to_row = |y: f64| -> u16 {
            let ratio =
                (y - y_bounds_aligned[0]) / (y_bounds_aligned[1] - y_bounds_aligned[0]);
            // Invert: larger Y values go to smaller row numbers
            let row = (1.0 - ratio) * (chart_inner_height - 1) as f64;
            row.clamp(0.0, chart_inner_height as f64 - 1.0) as u16
        };

        // Render each bin
        for bin in &bins {
            if bin.count == 0 {
                continue;
            }

            let density = bin.count as f64 / max_count as f64;
            let symbol = density_char(density);

            // Calculate bin boundaries in screen coordinates
            let x_start = x_to_col(bin.x_min);
            let x_end = x_to_col(bin.x_max);
            let y_start = y_to_row(bin.y_max); // Top of bin
            let y_end = y_to_row(bin.y_min); // Bottom of bin

            // Fill the bin area with the density character
            for row in y_start..=y_end {
                for col in x_start..=x_end {
                    let color = density_color(density);
                    let _ = f.buffer_mut().set_stringn(
                        col,
                        row,
                        symbol,
                        1,
                        Style::default().fg(color),
                    );
                }
            }
        }

        // Draw horizontal legend for density scale (low to high)
        let legend_y = 0u16;
        // Reverse order: low density to high density
        // 7-level non-linear scale: low values (0-0.2) have finer granularity
        let scale_chars = [(0.05, "·"), (0.2, "░"), (0.4, "▒"), (0.6, "▓"), (0.8, "█")];
        let legend_text = format!(" Max:{}", max_count);
        let legend_width = scale_chars.len() + legend_text.len();
        let legend_x = config.width.saturating_sub(legend_width as u16);

        // Draw density scale characters horizontally (low to high)
        for (i, (threshold, ch)) in scale_chars.iter().enumerate() {
            let col = legend_x + i as u16;
            let color = density_color(*threshold);
            let _ = f.buffer_mut().set_stringn(
                col,
                legend_y,
                ch,
                1,
                Style::default().fg(color),
            );
        }

        // Draw " Max:N" text after density chars
        let _ = f.buffer_mut().set_stringn(
            legend_x + scale_chars.len() as u16,
            legend_y,
            &legend_text,
            legend_text.len(),
            Style::default().fg(Color::White),
        );
    })?;

    // Print the buffer content to stdout
    let buffer = terminal.backend().buffer();
    render::print_buffer_to_stdout(buffer, config.width as usize);

    Ok(())
}

/// Get color based on density.
/// Heat gradient: black -> grey -> white -> yellow -> red
/// 7-level non-linear scale matching density_char
fn density_color(density: f64) -> Color {
    match density {
        d if d >= 0.8 => Color::Red,    // █ - highest
        d if d >= 0.6 => Color::Yellow, // ▓
        d if d >= 0.4 => Color::White,  // ▒
        d if d >= 0.2 => Color::Gray,   // ░
        d if d >= 0.05 => Color::Black, // · - low
        _ => Color::Black,              // (space) below 5%
    }
}

/// Compute optimal bin count using Freedman-Diaconis rule.
pub fn freedman_diaconis_bins(values: &[f64]) -> usize {
    if values.len() < 4 {
        return 10;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let q1_idx = (sorted.len() as f64 * 0.25).floor() as usize;
    let q3_idx = (sorted.len() as f64 * 0.75).floor() as usize;
    let q1 = sorted[q1_idx.min(sorted.len() - 1)];
    let q3 = sorted[q3_idx.min(sorted.len() - 1)];
    let iqr = q3 - q1;

    if iqr <= 0.0 {
        return 10;
    }

    let bin_width = 2.0 * iqr / (sorted.len() as f64).cbrt();
    let range = sorted.last().unwrap() - sorted.first().unwrap();

    (range / bin_width).ceil() as usize
}

/// Compute optimal bin count using Sturges' rule.
pub fn sturges_bins(n: usize) -> usize {
    (1.0 + (n as f64).log2()).ceil() as usize
}

/// Compute optimal bin count using square root rule.
pub fn sqrt_bins(n: usize) -> usize {
    (n as f64).sqrt().ceil() as usize
}
