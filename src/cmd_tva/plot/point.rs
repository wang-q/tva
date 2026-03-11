use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexMap;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, LegendPosition};
use ratatui::Terminal;

use crate::libs::io::reader;
use crate::libs::plot::{axis, render};
use crate::libs::tsv::fields::{parse_field_list_with_header, Header};
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::{Row, TsvRecord};
use crate::libs::tsv::split::TsvSplitter;

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
        .arg(Arg::new("y").short('y').long("y").required(true).help(
            "Y axis column(s), comma-separated (e.g., 'value1,value2' or '2,3,4')",
        ))
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
    let headers: Vec<Vec<u8>> = match header_line {
        Some(line) => TsvSplitter::new(&line, b'\t').map(|s| s.to_vec()).collect(),
        None => Vec::new(),
    };

    // Parse X column (single column)
    let x_idx = parse_column_index(x_col, &headers)?;

    // Parse Y columns (supports multiple columns like "2,3,4" or "sales2023,sales2024")
    let header_for_parsing = if headers.is_empty() {
        None
    } else {
        let field_names: Vec<String> = headers
            .iter()
            .map(|h| String::from_utf8_lossy(h).to_string())
            .collect();
        Some(Header::from_fields(field_names))
    };
    let y_indices =
        parse_field_list_with_header(y_col, header_for_parsing.as_ref(), '\t')
            .map_err(|e| anyhow::anyhow!("Invalid Y column spec: {}", e))?;
    if y_indices.is_empty() {
        return Err(anyhow::anyhow!("No valid Y columns specified"));
    }
    // Convert to 0-based indices
    let y_indices: Vec<usize> = y_indices.iter().map(|&i| i - 1).collect();

    let color_idx = color_col
        .map(|c| parse_column_index(c, &headers))
        .transpose()?;

    // Get Y column names for legend
    let y_names: Vec<String> = y_indices
        .iter()
        .map(|&idx| {
            headers
                .get(idx)
                .map(|h| String::from_utf8_lossy(h).to_string())
                .unwrap_or_else(|| format!("col{}", idx + 1))
        })
        .collect();

    let mut data: IndexMap<String, Vec<(f64, f64)>> = IndexMap::new();
    let mut record = TsvRecord::new();

    tsv_reader.for_each_record(|line| {
        record.parse_line(line, b'\t');

        // Parse X value
        let x_bytes = match record.get_bytes(x_idx + 1) {
            Some(b) => b,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Column {} not found in line", x_col),
                ));
            }
        };

        let x_val = match parse_float(x_bytes) {
            Some(v) => v,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot parse '{}' as number in column {}",
                        String::from_utf8_lossy(x_bytes),
                        x_col
                    ),
                ));
            }
        };

        // Get color group if specified
        let color_group = if let Some(idx) = color_idx {
            match record.get_bytes(idx + 1) {
                Some(b) => Some(String::from_utf8_lossy(b).to_string()),
                None => {
                    if ignore_errors {
                        return Ok(());
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Color column index {} exceeds number of fields {}",
                            idx + 1,
                            record.len()
                        ),
                    ));
                }
            }
        } else {
            None
        };

        // Parse each Y column
        for (y_idx, y_name) in y_indices.iter().zip(y_names.iter()) {
            let y_bytes = match record.get_bytes(y_idx + 1) {
                Some(b) => b,
                None => {
                    if ignore_errors {
                        continue;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Column {} not found in line", y_name),
                    ));
                }
            };

            let y_val = match parse_float(y_bytes) {
                Some(v) => v,
                None => {
                    if ignore_errors {
                        continue;
                    }
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Cannot parse '{}' as number in column {}",
                            String::from_utf8_lossy(y_bytes),
                            y_name
                        ),
                    ));
                }
            };

            // Build group key: if color is specified, use "color|y_name", else just "y_name"
            let group_key = match &color_group {
                Some(c) => format!("{}|{}", c, y_name),
                None => y_name.clone(),
            };

            data.entry(group_key).or_default().push((x_val, y_val));
        }

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

    // Build Y axis label from all Y column names
    let y_axis_label = if y_names.len() == 1 {
        y_names[0].clone()
    } else {
        format!("{} ({} series)", y_names.join(", "), y_names.len())
    };

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
        y_axis_label.as_bytes(),
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
        let color = render::get_color(i);

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

    // Generate axis labels using the shared axis utilities
    let x_labels: Vec<Span> =
        axis::generate_axis_labels(x_min, x_max, width as usize, 15, 3, 8)
            .into_iter()
            .map(Span::from)
            .collect();
    let y_labels: Vec<Span> =
        axis::generate_axis_labels(y_min, y_max, height as usize, 4, 3, 6)
            .into_iter()
            .map(Span::from)
            .collect();

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .title(Span::from(x_axis_label))
                .style(Style::default().fg(Color::Gray))
                .bounds([x_min, x_max])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title(Span::from(y_axis_label))
                .style(Style::default().fg(Color::Gray))
                .bounds([y_min, y_max])
                .labels(y_labels),
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
    render::print_buffer_to_stdout(buffer, width as usize);

    Ok(())
}
