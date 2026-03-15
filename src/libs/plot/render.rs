//! Terminal rendering utilities for plot commands.
//!
//! Provides color palette management and buffer-to-terminal rendering
//! with ANSI color support.

use colored::{ColoredString, Colorize};
use ratatui::buffer::{Buffer, Cell};
use ratatui::style::{Color, Modifier};

/// Default color palette for plot data series.
/// Ordered to provide good contrast in terminal environments.
pub const COLORS: &[Color] = &[
    Color::Cyan,
    Color::Green,
    Color::Yellow,
    Color::Magenta,
    Color::Blue,
    Color::Red,
];

/// Get a color from the palette by index.
/// Cycles through the palette if index exceeds palette size.
pub fn get_color(index: usize) -> Color {
    COLORS[index % COLORS.len()]
}

/// Parse marker type string to ratatui Marker.
///
/// Supported values:
/// - "dot" -> Marker::Dot
/// - "block" -> Marker::Block
/// - "braille" or any other value -> Marker::Braille (default)
pub fn parse_marker(marker_type: &str) -> ratatui::symbols::Marker {
    match marker_type {
        "dot" => ratatui::symbols::Marker::Dot,
        "block" => ratatui::symbols::Marker::Block,
        _ => ratatui::symbols::Marker::Braille,
    }
}

/// Print a ratatui buffer to stdout with ANSI colors.
///
/// This function converts the ratatui buffer content to colored terminal output,
/// grouping consecutive cells with the same style for efficient rendering.
pub fn print_buffer_to_stdout(buffer: &Buffer, cols: usize) {
    let contents = &buffer.content;
    let mut i: usize = 0;

    while i < contents.len() {
        let line = group_cells_by_color(&contents[i..(i + cols)])
            .iter()
            .map(|cells| {
                colorize(
                    &cells.iter().map(|cell| cell.symbol()).collect::<String>(),
                    cells[0].fg,
                    cells[0].modifier,
                )
                .to_string()
            })
            .collect::<String>();

        println!("{}", line.trim_end());
        i += cols;
    }
}

/// Group consecutive cells with the same style for efficient color rendering.
fn group_cells_by_color(cells: &[Cell]) -> Vec<Vec<Cell>> {
    let mut groups: Vec<Vec<Cell>> = Vec::new();
    let mut current_run: Vec<Cell> = Vec::new();

    for cell in cells {
        if current_run.is_empty() || (current_run[0].style() == cell.style()) {
            current_run.push(cell.clone());
            continue;
        }
        groups.push(current_run);
        current_run = vec![cell.clone()];
    }

    if !current_run.is_empty() {
        groups.push(current_run);
    }

    groups
}

/// Convert ratatui Color and Modifier to colored ColoredString.
fn colorize(string: &str, color: Color, modifier: Modifier) -> ColoredString {
    let string = match color {
        Color::Reset | Color::White => Colorize::normal(string),
        Color::Red => Colorize::red(string),
        Color::Blue => Colorize::blue(string),
        Color::Cyan => Colorize::cyan(string),
        Color::Green => Colorize::green(string),
        Color::Yellow => Colorize::yellow(string),
        Color::Magenta => Colorize::magenta(string),
        _ => Colorize::normal(string),
    };

    if modifier.is_empty() {
        return string;
    }

    match modifier {
        Modifier::DIM => Colorize::dimmed(string),
        _ => string,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_color() {
        assert_eq!(get_color(0), Color::Cyan);
        assert_eq!(get_color(1), Color::Green);
        assert_eq!(get_color(5), Color::Red);
        assert_eq!(get_color(6), Color::Cyan); // Cycles back
        assert_eq!(get_color(12), Color::Cyan); // Cycles back again
    }

    #[test]
    fn test_parse_marker() {
        assert_eq!(parse_marker("dot"), ratatui::symbols::Marker::Dot);
        assert_eq!(parse_marker("block"), ratatui::symbols::Marker::Block);
        assert_eq!(parse_marker("braille"), ratatui::symbols::Marker::Braille);
        assert_eq!(parse_marker("unknown"), ratatui::symbols::Marker::Braille); // Default
        assert_eq!(parse_marker(""), ratatui::symbols::Marker::Braille); // Default for empty
    }

    #[test]
    fn test_group_cells_by_color() {
        let cells = vec![Cell::new("a"), Cell::new("b"), Cell::new("c")];
        let groups = group_cells_by_color(&cells);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }

    #[test]
    fn test_group_cells_by_color_different_styles() {
        let mut cell1 = Cell::new("a");
        cell1.set_fg(Color::Red);
        let mut cell2 = Cell::new("b");
        cell2.set_fg(Color::Blue);
        let cells = vec![cell1, cell2];

        let groups = group_cells_by_color(&cells);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_group_cells_by_color_empty() {
        let cells: Vec<Cell> = vec![];
        let groups = group_cells_by_color(&cells);
        assert!(groups.is_empty());
    }
}
