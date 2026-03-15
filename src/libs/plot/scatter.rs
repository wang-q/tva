//! Scatter plot and line chart rendering for terminal-based visualization.

use anyhow::Result;
use indexmap::IndexMap;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, LegendPosition};
use ratatui::Terminal;

use super::axis;
use super::render;

/// Configuration for scatter/line chart rendering.
pub struct ScatterConfig {
    pub width: u16,
    pub height: u16,
    pub marker: Marker,
    pub is_line: bool,
    pub is_path: bool,
    pub draw_regression: bool,
    pub x_label: String,
    pub y_label: String,
}

impl ScatterConfig {
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
pub struct ScatterDataset {
    pub name: String,
    pub points: Vec<(f64, f64)>,
    pub color_idx: usize,
    pub is_regression: bool,
}

impl ScatterDataset {
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
pub fn process_scatter_data(
    data: IndexMap<String, Vec<(f64, f64)>>,
    is_line: bool,
    regression_data: Vec<(String, Vec<(f64, f64)>, usize)>,
) -> Vec<ScatterDataset> {
    let mut datasets: Vec<ScatterDataset> = Vec::new();

    for (i, (group, mut points)) in data.into_iter().enumerate() {
        if is_line {
            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }

        let name = if group.is_empty() {
            "data".to_string()
        } else {
            group
        };

        datasets.push(ScatterDataset::new(name, points, i));
    }

    for (name, points, color_idx) in regression_data {
        datasets.push(ScatterDataset::new_regression(name, points, color_idx));
    }

    datasets
}

/// Render a scatter plot or line chart to stdout.
pub fn render_scatter_chart(
    datasets: Vec<ScatterDataset>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    config: &ScatterConfig,
) -> Result<()> {
    let mut ratatui_datasets: Vec<Dataset> = Vec::new();

    for ds in &datasets {
        let color = render::get_color(ds.color_idx);

        let dataset = if ds.is_regression {
            // Regression lines always show their name (equation)
            Dataset::default()
                .name(ds.name.clone())
                .marker(Marker::Braille)
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .graph_type(GraphType::Line)
                .data(&ds.points)
        } else {
            let graph_type = if config.is_line || config.is_path {
                GraphType::Line
            } else {
                GraphType::Scatter
            };

            if config.draw_regression && !config.is_line && !config.is_path {
                // Show data point names even when showing regression
                Dataset::default()
                    .name(ds.name.clone())
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

    let x_labels_vec =
        axis::generate_axis_labels(x_min, x_max, config.width as usize, 4, 2, 4);
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

    let x_labels: Vec<Span> = x_labels_vec.into_iter().map(Span::from).collect();
    let y_labels: Vec<Span> = y_labels_vec.into_iter().map(Span::from).collect();

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
        });

    let backend = TestBackend::new(config.width, config.height);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let area = Rect::new(0, 0, config.width, config.height);
        f.render_widget(chart, area);
    })?;

    let buffer = terminal.backend().buffer();
    render::print_buffer_to_stdout(buffer, config.width as usize);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scatter_config_new() {
        let config = ScatterConfig::new(80, 24);
        assert_eq!(config.width, 80);
        assert_eq!(config.height, 24);
        assert_eq!(config.marker, Marker::Dot);
        assert!(!config.is_line);
        assert!(!config.is_path);
        assert!(!config.draw_regression);
    }

    #[test]
    fn test_scatter_config_with_marker() {
        let config = ScatterConfig::new(80, 24).with_marker(Marker::Braille);
        assert_eq!(config.marker, Marker::Braille);
    }

    #[test]
    fn test_scatter_config_with_line() {
        let config = ScatterConfig::new(80, 24).with_line();
        assert!(config.is_line);
    }

    #[test]
    fn test_scatter_config_with_path() {
        let config = ScatterConfig::new(80, 24).with_path();
        assert!(config.is_path);
    }

    #[test]
    fn test_scatter_config_with_regression() {
        let config = ScatterConfig::new(80, 24).with_regression();
        assert!(config.draw_regression);
    }

    #[test]
    fn test_scatter_config_with_labels() {
        let config = ScatterConfig::new(80, 24).with_labels("x-axis", "y-axis");
        assert_eq!(config.x_label, "x-axis");
        assert_eq!(config.y_label, "y-axis");
    }

    #[test]
    fn test_scatter_dataset_new() {
        let points = vec![(1.0, 2.0), (3.0, 4.0)];
        let ds = ScatterDataset::new("test", points.clone(), 0);
        assert_eq!(ds.name, "test");
        assert_eq!(ds.points, points);
        assert_eq!(ds.color_idx, 0);
        assert!(!ds.is_regression);
    }

    #[test]
    fn test_scatter_dataset_new_regression() {
        let points = vec![(1.0, 2.0), (3.0, 4.0)];
        let ds = ScatterDataset::new_regression("reg", points.clone(), 1);
        assert_eq!(ds.name, "reg");
        assert_eq!(ds.points, points);
        assert_eq!(ds.color_idx, 1);
        assert!(ds.is_regression);
    }

    #[test]
    fn test_process_scatter_data_basic() {
        let mut data: IndexMap<String, Vec<(f64, f64)>> = IndexMap::new();
        data.insert(
            "group1".to_string(),
            vec![(3.0, 1.0), (1.0, 2.0), (2.0, 3.0)],
        );

        let datasets = process_scatter_data(data, false, vec![]);

        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].name, "group1");
        assert_eq!(datasets[0].points, vec![(3.0, 1.0), (1.0, 2.0), (2.0, 3.0)]);
    }

    #[test]
    fn test_process_scatter_data_with_line_sorting() {
        let mut data: IndexMap<String, Vec<(f64, f64)>> = IndexMap::new();
        data.insert(
            "group1".to_string(),
            vec![(3.0, 1.0), (1.0, 2.0), (2.0, 3.0)],
        );

        let datasets = process_scatter_data(data, true, vec![]);

        assert_eq!(datasets.len(), 1);
        // Points should be sorted by x value
        assert_eq!(datasets[0].points, vec![(1.0, 2.0), (2.0, 3.0), (3.0, 1.0)]);
    }

    #[test]
    fn test_process_scatter_data_empty_group_name() {
        let mut data: IndexMap<String, Vec<(f64, f64)>> = IndexMap::new();
        data.insert("".to_string(), vec![(1.0, 2.0)]);

        let datasets = process_scatter_data(data, false, vec![]);

        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].name, "data"); // Empty name becomes "data"
    }

    #[test]
    fn test_process_scatter_data_with_regression() {
        let mut data: IndexMap<String, Vec<(f64, f64)>> = IndexMap::new();
        data.insert("group1".to_string(), vec![(1.0, 2.0)]);

        let regression_data =
            vec![("y = 2x".to_string(), vec![(0.0, 0.0), (1.0, 2.0)], 0)];
        let datasets = process_scatter_data(data, false, regression_data);

        assert_eq!(datasets.len(), 2);
        assert!(!datasets[0].is_regression);
        assert!(datasets[1].is_regression);
        assert_eq!(datasets[1].name, "y = 2x");
    }

    #[test]
    fn test_process_scatter_data_multiple_groups() {
        let mut data: IndexMap<String, Vec<(f64, f64)>> = IndexMap::new();
        data.insert("A".to_string(), vec![(1.0, 2.0)]);
        data.insert("B".to_string(), vec![(3.0, 4.0)]);
        data.insert("C".to_string(), vec![(5.0, 6.0)]);

        let datasets = process_scatter_data(data, false, vec![]);

        assert_eq!(datasets.len(), 3);
        assert_eq!(datasets[0].color_idx, 0);
        assert_eq!(datasets[1].color_idx, 1);
        assert_eq!(datasets[2].color_idx, 2);
    }
}
