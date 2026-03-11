use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::{ColoredString, Colorize};
use indexmap::IndexMap;
use ratatui::backend::TestBackend;
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, LegendPosition};
use ratatui::Terminal;

use crate::libs::io::reader;
use crate::libs::tsv::reader::TsvReader;

const COLORS: &[Color] = &[
    Color::Cyan,
    Color::Green,
    Color::Yellow,
    Color::Magenta,
    Color::Blue,
    Color::Red,
];

pub fn make_subcommand() -> Command {
    Command::new("point")
        .about("Draw a scatter plot or line chart")
        .after_help(include_str!("../../../docs/help/plot_point.md"))
        .arg(
            Arg::new("x")
                .short('x')
                .long("x")
                .required(true)
                .help("X axis column (1-based index or column name)"),
        )
        .arg(
            Arg::new("y")
                .short('y')
                .long("y")
                .required(true)
                .help("Y axis column (1-based index or column name)"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .help("Color grouping column (1-based index or column name)"),
        )
        .arg(
            Arg::new("line")
                .short('l')
                .long("line")
                .action(ArgAction::SetTrue)
                .help("Draw line chart (sort by x value)"),
        )
        .arg(
            Arg::new("path")
                .long("path")
                .action(ArgAction::SetTrue)
                .help("Draw path chart (connect points in original order, no sorting)"),
        )
        .arg(
            Arg::new("cols")
                .long("cols")
                .help("Chart width (characters or ratio like '0.8')"),
        )
        .arg(
            Arg::new("rows")
                .long("rows")
                .help("Chart height (characters or ratio like '0.6')"),
        )
        .arg(
            Arg::new("marker")
                .short('m')
                .long("marker")
                .help("Marker type: braille, dot, block")
                .default_value("braille"),
        )
        .arg(
            Arg::new("xlim")
                .long("xlim")
                .help("X axis limits (min,max)"),
        )
        .arg(
            Arg::new("ylim")
                .long("ylim")
                .help("Y axis limits (min,max)"),
        )
        .arg(
            Arg::new("ignore")
                .long("ignore")
                .action(ArgAction::SetTrue)
                .help("Ignore rows with non-numeric values"),
        )
        .arg(
            Arg::new("infile")
                .help("Input TSV file to process (default: stdin)")
                .index(1),
        )
}

pub fn execute(matches: &ArgMatches) -> Result<()> {
    let x_col = matches.get_one::<String>("x").unwrap();
    let y_col = matches.get_one::<String>("y").unwrap();
    let color_col = matches.get_one::<String>("color");
    let is_line = matches.get_flag("line");
    let is_path = matches.get_flag("path");
    let ignore_errors = matches.get_flag("ignore");
    let marker_type = matches.get_one::<String>("marker").unwrap();

    // --line and --path are mutually exclusive
    if is_line && is_path {
        return Err(anyhow::anyhow!(
            "Cannot use both --line and --path. Choose one."
        ));
    }

    let marker = match marker_type.as_str() {
        "dot" => Marker::Dot,
        "block" => Marker::Block,
        _ => Marker::Braille,
    };

    let infile = matches.get_one::<String>("infile");
    let input_reader = match infile {
        Some(path) => reader(path)?,
        None => reader("stdin")?,
    };

    let mut tsv_reader: TsvReader<_> = TsvReader::new(input_reader);

    // Read header (first line)
    let header_line = tsv_reader.read_header()?;
    let headers = match header_line {
        Some(line) => line.split(|&b| b == b'\t').map(|s| s.to_vec()).collect(),
        None => Vec::new(),
    };

    let x_idx = parse_column_index(x_col, &headers)?;
    let y_idx = parse_column_index(y_col, &headers)?;
    let color_idx = color_col
        .map(|c| parse_column_index(c, &headers))
        .transpose()?;

    let mut data: IndexMap<String, Vec<(f64, f64)>> = IndexMap::new();

    tsv_reader.for_each_record(|record| {
        let fields: Vec<&[u8]> = record.split(|&b| b == b'\t').collect();

        if x_idx >= fields.len() || y_idx >= fields.len() {
            return Ok(());
        }

        let x_val = match parse_float(fields[x_idx]) {
            Some(v) => v,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot parse '{}' as number in column {}",
                        String::from_utf8_lossy(fields[x_idx]),
                        x_col
                    ),
                ));
            }
        };

        let y_val = match parse_float(fields[y_idx]) {
            Some(v) => v,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot parse '{}' as number in column {}",
                        String::from_utf8_lossy(fields[y_idx]),
                        y_col
                    ),
                ));
            }
        };

        let group = if let Some(idx) = color_idx {
            if idx < fields.len() {
                String::from_utf8_lossy(fields[idx]).to_string()
            } else if ignore_errors {
                return Ok(());
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Color column index {} exceeds number of fields {}",
                        idx,
                        fields.len()
                    ),
                ));
            }
        } else {
            String::new()
        };

        data.entry(group).or_default().push((x_val, y_val));

        Ok(())
    })?;

    if data.is_empty() || data.values().all(|v| v.is_empty()) {
        return Err(anyhow::anyhow!("No valid data points to plot"));
    }

    let (x_min, x_max, y_min, y_max) = calculate_bounds(&data, matches)?;

    let (term_width, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
    let default_width = (term_width as f64 * 0.8) as u16;
    let default_height = (term_height as f64 * 0.8) as u16;

    let chart_width = parse_dimension(matches.get_one::<String>("cols"), default_width)?;
    let chart_height =
        parse_dimension(matches.get_one::<String>("rows"), default_height)?;

    render_chart(
        data,
        x_min,
        x_max,
        y_min,
        y_max,
        chart_width,
        chart_height,
        marker,
        is_line,
        is_path,
        &headers.get(x_idx).map(|h| h.as_slice()).unwrap_or(b"x"),
        &headers.get(y_idx).map(|h| h.as_slice()).unwrap_or(b"y"),
    )?;

    Ok(())
}

fn parse_column_index(col: &str, headers: &[Vec<u8>]) -> Result<usize> {
    if let Ok(idx) = col.parse::<usize>() {
        if idx == 0 {
            return Err(anyhow::anyhow!("Column index must be 1-based"));
        }
        if idx > headers.len() {
            return Err(anyhow::anyhow!(
                "Column index {} exceeds number of columns {}",
                idx,
                headers.len()
            ));
        }
        return Ok(idx - 1);
    }

    for (i, header) in headers.iter().enumerate() {
        if String::from_utf8_lossy(header) == col {
            return Ok(i);
        }
    }

    Err(anyhow::anyhow!("Column '{}' not found in headers", col))
}

fn parse_float(bytes: &[u8]) -> Option<f64> {
    crate::libs::number::fast_parse_f64(bytes)
}

fn calculate_bounds(
    data: &IndexMap<String, Vec<(f64, f64)>>,
    matches: &ArgMatches,
) -> Result<(f64, f64, f64, f64)> {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for points in data.values() {
        for (x, y) in points {
            x_min = x_min.min(*x);
            x_max = x_max.max(*x);
            y_min = y_min.min(*y);
            y_max = y_max.max(*y);
        }
    }

    if let Some(xlim) = matches.get_one::<String>("xlim") {
        let parts: Vec<&str> = xlim.split(',').collect();
        if parts.len() == 2 {
            x_min = parts[0].parse()?;
            x_max = parts[1].parse()?;
        }
    }

    if let Some(ylim) = matches.get_one::<String>("ylim") {
        let parts: Vec<&str> = ylim.split(',').collect();
        if parts.len() == 2 {
            y_min = parts[0].parse()?;
            y_max = parts[1].parse()?;
        }
    }

    if x_min == x_max {
        x_min -= 1.0;
        x_max += 1.0;
    }
    if y_min == y_max {
        y_min -= 1.0;
        y_max += 1.0;
    }

    Ok((x_min, x_max, y_min, y_max))
}

/// Calculate a "nice" midpoint for axis labels.
/// Rounds to a reasonable number of significant digits based on the range.
fn nice_midpoint(min: f64, max: f64) -> f64 {
    let mid = (min + max) / 2.0;
    let range = max - min;

    if range == 0.0 {
        return mid;
    }

    // Determine appropriate precision based on range magnitude
    let magnitude = range.log10().floor() as i32;
    let precision = if magnitude >= 2 {
        0 // Whole numbers for large ranges
    } else if magnitude >= 0 {
        1 // 1 decimal place
    } else if magnitude >= -2 {
        2 // 2 decimal places
    } else {
        3 // 3 decimal places for very small ranges
    };

    let factor = 10f64.powi(precision);
    (mid * factor).round() / factor
}

fn parse_dimension(value: Option<&String>, default: u16) -> Result<u16> {
    match value {
        None => Ok(default),
        Some(v) => {
            if v.contains('.') {
                let ratio: f64 = v.parse()?;
                let result = (default as f64 * ratio).round() as u16;
                Ok(result.max(10)) // Minimum 10 characters
            } else {
                let result: u16 = v.parse()?;
                Ok(result.max(10)) // Minimum 10 characters
            }
        }
    }
}

fn render_chart(
    data: IndexMap<String, Vec<(f64, f64)>>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    width: u16,
    height: u16,
    marker: Marker,
    is_line: bool,
    is_path: bool,
    x_label: &[u8],
    y_label: &[u8],
) -> Result<()> {
    // Prepare all data first, then create datasets
    let mut all_data: Vec<(String, Vec<(f64, f64)>)> = Vec::new();

    for (group, points) in data {
        let mut sorted_points = points;
        // --line: sort by x value (geom_line behavior)
        // --path: keep original order (geom_path behavior)
        if is_line {
            sorted_points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }
        all_data.push((group, sorted_points));
    }

    let mut datasets: Vec<Dataset> = Vec::new();

    for (i, (group, points)) in all_data.iter().enumerate() {
        let color = COLORS[i % COLORS.len()];

        let dataset = Dataset::default()
            .name(if group.is_empty() { "data" } else { group })
            .marker(marker)
            .style(Style::default().fg(color))
            .graph_type(if is_line || is_path {
                GraphType::Line
            } else {
                GraphType::Scatter
            })
            .data(points.as_slice());

        datasets.push(dataset);
    }

    let x_axis_label = String::from_utf8_lossy(x_label).to_string();
    let y_axis_label = String::from_utf8_lossy(y_label).to_string();

    // Calculate nice mid-point for axis labels
    let x_mid = nice_midpoint(x_min, x_max);
    let y_mid = nice_midpoint(y_min, y_max);

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .title(Span::from(x_axis_label))
                .style(Style::default().fg(Color::Gray))
                .bounds([x_min, x_max])
                .labels(vec![
                    Span::from(format!("{:.2}", x_min)),
                    Span::from(format!("{:.2}", x_mid)),
                    Span::from(format!("{:.2}", x_max)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(Span::from(y_axis_label))
                .style(Style::default().fg(Color::Gray))
                .bounds([y_min, y_max])
                .labels(vec![
                    Span::from(format!("{:.2}", y_min)),
                    Span::from(format!("{:.2}", y_mid)),
                    Span::from(format!("{:.2}", y_max)),
                ]),
        )
        .legend_position(Some(LegendPosition::TopRight));

    // Use TestBackend to render to an off-screen buffer, then print to stdout
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let area = Rect::new(0, 0, width, height);
        f.render_widget(chart, area);
    })?;

    // Print the buffer content to stdout with colors
    let buffer = terminal.backend().buffer();
    print_buffer_to_stdout(buffer, width as usize);

    Ok(())
}

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

fn print_buffer_to_stdout(buffer: &Buffer, cols: usize) {
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
