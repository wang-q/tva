use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexMap;

use crate::libs::io::reader;
use crate::libs::plot::{
    axis,
    chart::{process_data, render_chart, ChartConfig},
    regression, render,
};
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
                .default_value("1.0")
                .help("Chart width (characters or ratio like '0.8')"),
        )
        .arg(
            Arg::new("rows")
                .long("rows")
                .default_value("1.0")
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
            Arg::new("regression")
                .short('r')
                .long("regression")
                .action(ArgAction::SetTrue)
                .help("Draw regression line (linear fit)"),
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
    let draw_regression = matches.get_flag("regression");
    let ignore_errors = matches.get_flag("ignore");
    let marker_type = matches.get_one::<String>("marker").unwrap();

    // --line and --path are mutually exclusive
    if is_line && is_path {
        return Err(anyhow::anyhow!(
            "Cannot use both --line and --path. Choose one."
        ));
    }

    // --regression cannot be used with --line or --path
    if draw_regression && (is_line || is_path) {
        return Err(anyhow::anyhow!(
            "Cannot use --regression with --line or --path"
        ));
    }

    let marker = render::parse_marker(marker_type);

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

    // Build header for field parsing
    let header_for_parsing = if headers.is_empty() {
        None
    } else {
        let field_names: Vec<String> = headers
            .iter()
            .map(|h| String::from_utf8_lossy(h).to_string())
            .collect();
        Some(Header::from_fields(field_names))
    };

    // Parse X column (single column)
    let x_indices =
        parse_field_list_with_header(x_col, header_for_parsing.as_ref(), '\t')
            .map_err(|e| anyhow::anyhow!("Invalid X column spec: {}", e))?;
    if x_indices.is_empty() {
        return Err(anyhow::anyhow!("No valid X column specified"));
    }
    if x_indices.len() > 1 {
        return Err(anyhow::anyhow!(
            "X column must be a single column, got {} columns",
            x_indices.len()
        ));
    }
    let x_idx = x_indices[0] - 1; // Convert to 0-based

    // Parse Y columns (supports multiple columns like "2,3,4" or "sales2023,sales2024")
    let y_indices =
        parse_field_list_with_header(y_col, header_for_parsing.as_ref(), '\t')
            .map_err(|e| anyhow::anyhow!("Invalid Y column spec: {}", e))?;
    if y_indices.is_empty() {
        return Err(anyhow::anyhow!("No valid Y columns specified"));
    }
    // Convert to 0-based indices
    let y_indices: Vec<usize> = y_indices.iter().map(|&i| i - 1).collect();

    // Parse color column (single column, optional)
    let color_idx = match color_col {
        Some(c) => {
            let c_indices =
                parse_field_list_with_header(c, header_for_parsing.as_ref(), '\t')
                    .map_err(|e| anyhow::anyhow!("Invalid color column spec: {}", e))?;
            if c_indices.is_empty() {
                return Err(anyhow::anyhow!("No valid color column specified"));
            }
            if c_indices.len() > 1 {
                return Err(anyhow::anyhow!(
                    "Color column must be a single column, got {} columns",
                    c_indices.len()
                ));
            }
            Some(c_indices[0] - 1) // Convert to 0-based
        }
        None => None,
    };

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

            // Build group key:
            // - If color is specified and multiple Y columns: use "color|y_name"
            // - If color is specified and single Y column: use "color"
            // - If no color: use "y_name"
            let group_key = match &color_group {
                Some(c) => {
                    if y_indices.len() > 1 {
                        format!("{}|{}", c, y_name)
                    } else {
                        c.clone()
                    }
                }
                None => y_name.clone(),
            };

            data.entry(group_key).or_default().push((x_val, y_val));
        }

        Ok(())
    })?;

    if data.is_empty() || data.values().all(|v| v.is_empty()) {
        return Err(anyhow::anyhow!("No valid data points to plot"));
    }

    // Calculate bounds from all data points (automatic)
    let all_points: Vec<(f64, f64)> = data.values().flatten().copied().collect();
    let (x_min, x_max, y_min, y_max) =
        axis::calculate_bounds(all_points.iter().copied());

    // Get chart dimensions
    // Reserve 1 row for the terminal prompt at the bottom
    let (term_width, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
    let available_height = term_height.saturating_sub(1).max(10);
    let chart_width = parse_chart_dimension(
        matches.get_one::<String>("cols"),
        term_width,
        (term_width as f64 * 0.7) as u16,
    )?;
    let chart_height = parse_chart_dimension(
        matches.get_one::<String>("rows"),
        available_height,
        (available_height as f64 * 0.7) as u16,
    )?;

    // Build Y axis label from all Y column names
    let y_axis_label = if y_names.len() == 1 {
        y_names[0].clone()
    } else {
        format!("{} ({} series)", y_names.join(", "), y_names.len())
    };

    // Prepare regression data if requested
    let mut regression_data: Vec<(String, Vec<(f64, f64)>, usize)> = Vec::new();
    if draw_regression {
        for (i, (group, points)) in data.iter().enumerate() {
            if let Some((slope, intercept)) = regression::calculate_regression(points) {
                let reg_points = regression::generate_regression_points(
                    slope, intercept, x_min, x_max,
                );
                let equation = regression::format_regression_equation(slope, intercept);
                let reg_name = format!(
                    "{} ({})",
                    if group.is_empty() { "data" } else { group },
                    equation
                );
                regression_data.push((reg_name, reg_points, i));
            }
        }
    }

    // Process data into datasets
    let datasets = process_data(data, is_line, regression_data);

    // Build chart configuration
    let config = ChartConfig::new(chart_width, chart_height)
        .with_marker(marker)
        .with_labels(
            String::from_utf8_lossy(
                headers.get(x_idx).map(|h| h.as_slice()).unwrap_or(b"x"),
            ),
            y_axis_label,
        );

    let config = if is_line {
        config.with_line()
    } else if is_path {
        config.with_path()
    } else {
        config
    };

    // Render the chart
    render_chart(datasets, x_min, x_max, y_min, y_max, &config)?;

    Ok(())
}

/// Parse chart dimension with support for:
/// - Absolute values (e.g., "80" for 80 characters)
/// - Ratios relative to terminal size (e.g., "0.8" for 80% of terminal)
/// - Ratios > 1.0 to fill terminal (e.g., "1.0" for 100% of terminal)
fn parse_chart_dimension(
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

fn parse_float(bytes: &[u8]) -> Option<f64> {
    crate::libs::number::fast_parse_f64(bytes)
}
