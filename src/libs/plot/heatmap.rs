//! Heatmap rendering for 2D binning visualization.
//!
//! Provides terminal-based heatmap rendering using character density
//! to represent data density in 2D binned data.

use anyhow::Result;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};
use ratatui::Terminal;

use super::axis;
use super::binning::{compute_bins_2d, Bin2dConfig};
use super::render;

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

/// Render a 2D binning heatmap to stdout.
pub fn render_heatmap(
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
        let y_label_width =
            y_labels_vec.iter().map(|l| l.len()).max().unwrap_or(1) as u16;
        let chart_inner_x = y_label_width + 1; // Y labels + axis line
        let _chart_inner_y = 0; // Top margin
        let chart_inner_width = config.width.saturating_sub(chart_inner_x + 1);
        let chart_inner_height = config.height.saturating_sub(2);

        // Map data coordinates to screen coordinates
        let x_to_col = |x: f64| -> u16 {
            let ratio =
                (x - x_bounds_aligned[0]) / (x_bounds_aligned[1] - x_bounds_aligned[0]);
            let col = chart_inner_x + (ratio * (chart_inner_width - 1) as f64) as u16;
            col.clamp(chart_inner_x, config.width - 1)
        };

        let y_to_row = |y: f64| -> u16 {
            let ratio =
                (y - y_bounds_aligned[0]) / (y_bounds_aligned[1] - y_bounds_aligned[0]);
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

            let x_start = x_to_col(bin.x_min);
            let x_end = x_to_col(bin.x_max);
            let y_start = y_to_row(bin.y_max);
            let y_end = y_to_row(bin.y_min);

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

        // Draw horizontal legend for density scale
        let legend_y = 0u16;
        let scale_chars = [(0.05, "·"), (0.2, "░"), (0.4, "▒"), (0.6, "▓"), (0.8, "█")];
        let legend_text = format!(" Max:{}", max_count);
        let legend_width = scale_chars.len() + legend_text.len();
        let legend_x = config.width.saturating_sub(legend_width as u16);

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
