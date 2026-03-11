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
