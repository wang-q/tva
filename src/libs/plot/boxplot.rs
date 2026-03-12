//! Box plot (box-and-whisker) rendering for terminal-based visualization.

use anyhow::Result;
use indexmap::IndexMap;
use ratatui::backend::TestBackend;
use ratatui::style::{Color, Style};
use ratatui::Terminal;

use super::axis;
use super::render;
use super::stats::BoxStats;

/// Configuration for box plot rendering.
pub struct BoxPlotConfig {
    pub width: u16,
    pub height: u16,
    pub show_outliers: bool,
}

impl BoxPlotConfig {
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
pub fn render_boxplot<T: BoxStatsRender>(
    box_data: IndexMap<String, T>,
    y_min: f64,
    y_max: f64,
    config: &BoxPlotConfig,
) -> Result<()> {
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

    let y_labels_vec =
        axis::generate_axis_labels(y_min, y_max, config.height as usize, 4, 2, 4);

    let y_bounds_aligned = if y_labels_vec.len() >= 2 {
        let first: f64 = y_labels_vec.first().unwrap().parse().unwrap_or(y_min);
        let last: f64 = y_labels_vec.last().unwrap().parse().unwrap_or(y_max);
        [first, last]
    } else {
        [y_min, y_max]
    };

    let max_label_len = y_labels_vec.iter().map(|l| l.len()).max().unwrap_or(1);
    let y_axis_width = (max_label_len as u16).max(1);

    let backend = TestBackend::new(config.width, config.height);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let num_boxes = boxes.len();
        let chart_width = config.width.saturating_sub(y_axis_width);
        let box_width = if num_boxes > 0 {
            (chart_width as usize / num_boxes).max(8)
        } else {
            10
        };
        let box_width = box_width.min(20) as u16;

        let chart_height = config.height.saturating_sub(2);
        let y_to_row = |y: f64| -> u16 {
            let ratio =
                (y - y_bounds_aligned[0]) / (y_bounds_aligned[1] - y_bounds_aligned[0]);
            let row = (1.0 - ratio) * (chart_height - 1) as f64;
            row.clamp(0.0, chart_height as f64 - 1.0) as u16
        };

        for (i, box_data) in boxes.iter().enumerate() {
            let x_center = y_axis_width + (i as u16 * box_width) + (box_width / 2);
            let color = render::get_color(box_data.color_idx);

            let y_min_row = y_to_row(box_data.min);
            let y_q1_row = y_to_row(box_data.q1);
            let y_median_row = y_to_row(box_data.median);
            let y_q3_row = y_to_row(box_data.q3);
            let y_max_row = y_to_row(box_data.max);

            let (top_whisker, bottom_whisker) =
                (y_max_row.min(y_min_row), y_max_row.max(y_min_row));
            let (top_box, bottom_box) = (y_q3_row.min(y_q1_row), y_q3_row.max(y_q1_row));

            // Draw top whisker
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

            // Draw bottom whisker
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

            // Draw box
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

            // Draw group name
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

        // Draw Y axis line
        for row in 0..chart_height {
            let _ = f.buffer_mut().set_stringn(
                y_axis_width,
                row,
                "│",
                1,
                Style::default().fg(Color::Gray),
            );
        }

        // Draw Y axis labels
        for (i, label) in y_labels_vec.iter().enumerate() {
            if y_labels_vec.len() <= 1 {
                continue;
            }
            let ratio = 1.0 - (i as f64 / (y_labels_vec.len() - 1) as f64);
            let row = (ratio * (chart_height - 1) as f64) as u16;
            let row = row.min(chart_height - 1);
            let label_trimmed = if label.len() > y_axis_width as usize {
                &label[..y_axis_width as usize]
            } else {
                label
            };
            let x_offset = y_axis_width.saturating_sub(label_trimmed.len() as u16);
            let _ = f.buffer_mut().set_stringn(
                x_offset,
                row,
                label_trimmed,
                label_trimmed.len(),
                Style::default().fg(Color::Gray),
            );
        }

        // Draw X axis line
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
        let _ = f.buffer_mut().set_stringn(
            y_axis_width,
            x_axis_row,
            "├",
            1,
            Style::default().fg(Color::Gray),
        );
    })?;

    let buffer = terminal.backend().buffer();
    render::print_buffer_to_stdout(buffer, config.width as usize);

    Ok(())
}
