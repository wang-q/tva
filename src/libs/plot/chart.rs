//! Chart rendering utilities for terminal-based plotting.

use anyhow::Result;
use indexmap::IndexMap;
use ratatui::backend::TestBackend;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, LegendPosition};
use ratatui::Terminal;

use super::axis;
use super::render;
use super::stats::BoxStats;

/// Configuration for chart rendering.
pub struct ChartConfig {
    pub width: u16,
    pub height: u16,
    pub marker: Marker,
    pub is_line: bool,
    pub is_path: bool,
    pub draw_regression: bool,
    pub x_label: String,
    pub y_label: String,
}

impl ChartConfig {
    /// Create a new chart configuration with default settings.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            marker: Marker::Dot,
            is_line: false,
            is_path: false,
            draw_regression: false,
            x_label: String::new(),
            y_label: String::new(),
        }
    }

    /// Set the marker type.
    pub fn with_marker(mut self, marker: Marker) -> Self {
        self.marker = marker;
        self
    }

    /// Enable line mode (sorts points by x).
    pub fn with_line(mut self) -> Self {
        self.is_line = true;
        self
    }

    /// Enable path mode (keeps original order).
    pub fn with_path(mut self) -> Self {
        self.is_path = true;
        self
    }

    /// Enable regression line drawing.
    pub fn with_regression(mut self) -> Self {
        self.draw_regression = true;
        self
    }

    /// Set axis labels.
    pub fn with_labels(mut self, x: impl Into<String>, y: impl Into<String>) -> Self {
        self.x_label = x.into();
        self.y_label = y.into();
        self
    }
}

/// A dataset with its associated metadata.
pub struct PlotDataset {
    pub name: String,
    pub points: Vec<(f64, f64)>,
    pub color_idx: usize,
    pub is_regression: bool,
}

impl PlotDataset {
    /// Create a new regular dataset.
    pub fn new(
        name: impl Into<String>,
        points: Vec<(f64, f64)>,
        color_idx: usize,
    ) -> Self {
        Self {
            name: name.into(),
            points,
            color_idx,
            is_regression: false,
        }
    }

    /// Create a new regression line dataset.
    pub fn new_regression(
        name: impl Into<String>,
        points: Vec<(f64, f64)>,
        color_idx: usize,
    ) -> Self {
        Self {
            name: name.into(),
            points,
            color_idx,
            is_regression: true,
        }
    }
}

/// Process raw data into datasets ready for rendering.
/// Handles sorting for line mode and prepares regression data.
pub fn process_data(
    data: IndexMap<String, Vec<(f64, f64)>>,
    is_line: bool,
    regression_data: Vec<(String, Vec<(f64, f64)>, usize)>,
) -> Vec<PlotDataset> {
    let mut datasets: Vec<PlotDataset> = Vec::new();

    for (i, (group, mut points)) in data.into_iter().enumerate() {
        // --line: sort by x value (geom_line behavior)
        // --path: keep original order (geom_path behavior)
        if is_line {
            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }

        let name = if group.is_empty() {
            "data".to_string()
        } else {
            group
        };

        datasets.push(PlotDataset::new(name, points, i));
    }

    // Add regression line datasets
    for (name, points, color_idx) in regression_data {
        datasets.push(PlotDataset::new_regression(name, points, color_idx));
    }

    datasets
}

/// Render a chart to stdout.
pub fn render_chart(
    datasets: Vec<PlotDataset>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    config: &ChartConfig,
) -> Result<()> {
    // Convert PlotDatasets to ratatui Datasets
    let mut ratatui_datasets: Vec<Dataset> = Vec::new();

    for ds in &datasets {
        let color = render::get_color(ds.color_idx);

        let dataset = if ds.is_regression {
            // Regression line dataset
            Dataset::default()
                .name(ds.name.clone())
                .marker(Marker::Braille)
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .graph_type(GraphType::Line)
                .data(&ds.points)
        } else {
            // Regular data dataset
            let graph_type = if config.is_line || config.is_path {
                GraphType::Line
            } else {
                GraphType::Scatter
            };

            // When showing regression lines, don't add name to scatter datasets
            // so only the regression equations appear in the legend
            if config.draw_regression && !config.is_line && !config.is_path {
                Dataset::default()
                    .marker(config.marker)
                    .style(Style::default().fg(color))
                    .graph_type(graph_type)
                    .data(&ds.points)
            } else {
                Dataset::default()
                    .name(ds.name.clone())
                    .marker(config.marker)
                    .style(Style::default().fg(color))
                    .graph_type(graph_type)
                    .data(&ds.points)
            }
        };

        ratatui_datasets.push(dataset);
    }

    // Generate axis labels and compute aligned bounds
    let x_labels_vec =
        axis::generate_axis_labels(x_min, x_max, config.width as usize, 10, 2, 4);
    let y_labels_vec =
        axis::generate_axis_labels(y_min, y_max, config.height as usize, 4, 2, 4);

    // Compute aligned bounds from the generated labels
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

    let x_labels: Vec<Span> = x_labels_vec.into_iter().map(Span::from).collect();
    let y_labels: Vec<Span> = y_labels_vec.into_iter().map(Span::from).collect();

    // Only show legend if we have more than one dataset
    let has_multiple_datasets = ratatui_datasets.len() > 1;

    let chart = Chart::new(ratatui_datasets)
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
        )
        .legend_position(if has_multiple_datasets {
            Some(LegendPosition::TopRight)
        } else {
            None
        })
        .hidden_legend_constraints((Constraint::Min(0), Constraint::Min(0)));

    // Use TestBackend to render to an off-screen buffer, then print to stdout
    let backend = TestBackend::new(config.width, config.height);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let area = Rect::new(0, 0, config.width, config.height);
        f.render_widget(chart, area);
    })?;

    // Print the buffer content to stdout with colors
    let buffer = terminal.backend().buffer();
    render::print_buffer_to_stdout(buffer, config.width as usize);

    Ok(())
}

/// Configuration for box plot rendering.
pub struct ChartConfigBox {
    pub width: u16,
    pub height: u16,
    pub show_outliers: bool,
}

impl ChartConfigBox {
    /// Create a new box plot configuration with default settings.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            show_outliers: false,
        }
    }

    /// Enable outlier display.
    pub fn with_outliers(mut self, enable: bool) -> Self {
        self.show_outliers = enable;
        self
    }
}

/// Box plot data for a single group
pub struct BoxPlotData {
    pub name: String,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub outliers: Vec<f64>,
    pub color_idx: usize,
}

/// Box plot statistics trait for rendering
pub trait BoxStatsRender {
    fn min(&self) -> f64;
    fn q1(&self) -> f64;
    fn median(&self) -> f64;
    fn q3(&self) -> f64;
    fn max(&self) -> f64;
    fn outliers(&self) -> &[f64];
}

// Implement BoxStatsRender for stats::BoxStats
impl BoxStatsRender for BoxStats {
    fn min(&self) -> f64 {
        self.min
    }
    fn q1(&self) -> f64 {
        self.q1
    }
    fn median(&self) -> f64 {
        self.median
    }
    fn q3(&self) -> f64 {
        self.q3
    }
    fn max(&self) -> f64 {
        self.max
    }
    fn outliers(&self) -> &[f64] {
        &self.outliers
    }
}

/// Render a box plot to stdout.
pub fn render_chart_box<T: BoxStatsRender>(
    box_data: IndexMap<String, T>,
    y_min: f64,
    y_max: f64,
    config: &ChartConfigBox,
) -> Result<()> {
    // Convert to BoxPlotData
    let mut boxes: Vec<BoxPlotData> = Vec::new();
    for (i, (name, stats)) in box_data.iter().enumerate() {
        boxes.push(BoxPlotData {
            name: name.clone(),
            min: stats.min(),
            q1: stats.q1(),
            median: stats.median(),
            q3: stats.q3(),
            max: stats.max(),
            outliers: if config.show_outliers {
                stats.outliers().to_vec()
            } else {
                Vec::new()
            },
            color_idx: i,
        });
    }

    // Generate Y axis labels
    let y_labels_vec =
        axis::generate_axis_labels(y_min, y_max, config.height as usize, 4, 2, 4);

    let y_bounds_aligned = if y_labels_vec.len() >= 2 {
        let first: f64 = y_labels_vec.first().unwrap().parse().unwrap_or(y_min);
        let last: f64 = y_labels_vec.last().unwrap().parse().unwrap_or(y_max);
        [first, last]
    } else {
        [y_min, y_max]
    };

    // Use TestBackend to render
    let backend = TestBackend::new(config.width, config.height);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        // Calculate layout
        let num_boxes = boxes.len();
        let y_axis_width = 6u16;
        let chart_width = config.width.saturating_sub(y_axis_width);
        let box_width = if num_boxes > 0 {
            (chart_width as usize / num_boxes).max(8)
        } else {
            10
        };
        let box_width = box_width.min(20) as u16;

        // Map values to Y coordinates (chart area is config.height - 2 for labels)
        // Y values increase upward, so larger values map to smaller row numbers (top of screen)
        let chart_height = config.height.saturating_sub(2);
        let y_to_row = |y: f64| -> u16 {
            let ratio =
                (y - y_bounds_aligned[0]) / (y_bounds_aligned[1] - y_bounds_aligned[0]);
            // Invert: larger Y values go to smaller row numbers (top of screen)
            let row = (1.0 - ratio) * (chart_height - 1) as f64;
            row.clamp(0.0, chart_height as f64 - 1.0) as u16
        };

        // Render each box
        for (i, box_data) in boxes.iter().enumerate() {
            let x_center = y_axis_width + (i as u16 * box_width) + (box_width / 2);
            let color = render::get_color(box_data.color_idx);

            let y_min_row = y_to_row(box_data.min);
            let y_q1_row = y_to_row(box_data.q1);
            let y_median_row = y_to_row(box_data.median);
            let y_q3_row = y_to_row(box_data.q3);
            let y_max_row = y_to_row(box_data.max);

            // Ensure proper ordering (rows go from top to bottom, so max < min in row numbers)
            let (top_whisker, bottom_whisker) =
                (y_max_row.min(y_min_row), y_max_row.max(y_min_row));
            let (top_box, bottom_box) = (y_q3_row.min(y_q1_row), y_q3_row.max(y_q1_row));

            // Draw top whisker (from max to Q3)
            for row in top_whisker..=top_box {
                let symbol = if row == y_median_row { "┼" } else { "│" };
                let _ = f.buffer_mut().set_stringn(
                    x_center,
                    row,
                    symbol,
                    1,
                    Style::default().fg(color),
                );
            }

            // Draw bottom whisker (from Q1 to min)
            for row in bottom_box..=bottom_whisker {
                let symbol = if row == y_median_row { "┼" } else { "│" };
                let _ = f.buffer_mut().set_stringn(
                    x_center,
                    row,
                    symbol,
                    1,
                    Style::default().fg(color),
                );
            }

            // Draw box (Q1 to Q3)
            let box_left = x_center.saturating_sub(1);
            let box_right = (x_center + 1).min(config.width - 1);
            for row in top_box..=bottom_box {
                for col in box_left..=box_right {
                    let is_median_row = row == y_median_row;
                    let is_center = col == x_center;
                    let symbol = if is_median_row && is_center {
                        "┼"
                    } else if is_median_row {
                        "─"
                    } else {
                        "█"
                    };
                    let _ = f.buffer_mut().set_stringn(
                        col,
                        row,
                        symbol,
                        1,
                        Style::default().fg(color),
                    );
                }
            }

            // Draw whisker ends
            let _ = f.buffer_mut().set_stringn(
                x_center.saturating_sub(1),
                top_whisker,
                "─┬─",
                3,
                Style::default().fg(color),
            );
            let _ = f.buffer_mut().set_stringn(
                x_center.saturating_sub(1),
                bottom_whisker,
                "─┴─",
                3,
                Style::default().fg(color),
            );

            // Draw group name below chart
            let name = if box_data.name.len() > box_width as usize {
                format!("{}...", &box_data.name[..box_width as usize - 3])
            } else {
                box_data.name.clone()
            };
            let name_x = x_center.saturating_sub(name.len() as u16 / 2);
            let _ = f.buffer_mut().set_stringn(
                name_x,
                config.height - 1,
                &name,
                name.len(),
                Style::default().fg(color),
            );

            // Draw outliers
            if config.show_outliers {
                for outlier in &box_data.outliers {
                    let y_outlier = y_to_row(*outlier);
                    let _ = f.buffer_mut().set_stringn(
                        x_center,
                        y_outlier,
                        "•",
                        1,
                        Style::default().fg(color),
                    );
                }
            }
        }

        // Draw Y axis line (vertical)
        for row in 0..chart_height {
            let _ = f.buffer_mut().set_stringn(
                y_axis_width,
                row,
                "│",
                1,
                Style::default().fg(Color::Gray),
            );
        }

        // Draw Y axis labels (top to bottom: max to min)
        for (i, label) in y_labels_vec.iter().enumerate() {
            if y_labels_vec.len() <= 1 {
                continue;
            }
            // Invert: i=0 (min value) should be at bottom, i=n-1 (max value) at top
            let ratio = 1.0 - (i as f64 / (y_labels_vec.len() - 1) as f64);
            let row = (ratio * (chart_height - 1) as f64) as u16;
            let row = row.min(chart_height - 1);
            // Right-align the label
            let label_trimmed = if label.len() > y_axis_width as usize - 1 {
                &label[..y_axis_width as usize - 1]
            } else {
                label
            };
            let x_offset = y_axis_width.saturating_sub(label_trimmed.len() as u16 + 1);
            let _ = f.buffer_mut().set_stringn(
                x_offset,
                row,
                label_trimmed,
                label_trimmed.len(),
                Style::default().fg(Color::Gray),
            );
        }

        // Draw X axis line (horizontal at bottom)
        let x_axis_row = chart_height;
        for col in y_axis_width..config.width {
            let _ = f.buffer_mut().set_stringn(
                col,
                x_axis_row,
                "─",
                1,
                Style::default().fg(Color::Gray),
            );
        }
        // Draw corner
        let _ = f.buffer_mut().set_stringn(
            y_axis_width,
            x_axis_row,
            "├",
            1,
            Style::default().fg(Color::Gray),
        );
    })?;

    // Print the buffer content to stdout
    let buffer = terminal.backend().buffer();
    render::print_buffer_to_stdout(buffer, config.width as usize);

    Ok(())
}
