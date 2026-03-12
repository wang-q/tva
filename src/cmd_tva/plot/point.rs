use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::libs::io::reader;
use crate::libs::plot::{
    axis, build_header, load_scatter_data, parse_chart_dimension, parse_columns,
    parse_single_column, read_headers, regression, render,
    scatter::{process_scatter_data, render_scatter_chart, ScatterConfig},
};
use crate::libs::tsv::reader::TsvReader;

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
                .default_value("80")
                .help("Chart width (characters or ratio like '0.8')"),
        )
        .arg(
            Arg::new("rows")
                .long("rows")
                .default_value("24")
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

    // Validate flags
    if is_line && is_path {
        return Err(anyhow::anyhow!(
            "Cannot use both --line and --path. Choose one."
        ));
    }
    if draw_regression && (is_line || is_path) {
        return Err(anyhow::anyhow!(
            "Cannot use --regression with --line or --path"
        ));
    }

    let marker = render::parse_marker(marker_type);

    // Open input
    let infile = matches.get_one::<String>("infile");
    let input_reader = match infile {
        Some(path) => reader(path)?,
        None => reader("stdin")?,
    };

    let mut tsv_reader: TsvReader<_> = TsvReader::new(input_reader);

    // Read headers
    let headers = read_headers(&mut tsv_reader)?;
    let header_for_parsing = build_header(&headers);

    // Parse X column
    let (x_idx, x_name) =
        parse_single_column(x_col, header_for_parsing.as_ref(), &headers)?;

    // Parse Y columns
    let y_spec = parse_columns(y_col, header_for_parsing.as_ref(), &headers)?;
    let y_indices = y_spec.indices;
    let y_names = y_spec.names;

    // Parse color column
    let color_idx = match color_col {
        Some(c) => {
            let (idx, _) =
                parse_single_column(c, header_for_parsing.as_ref(), &headers)?;
            Some(idx)
        }
        None => None,
    };

    // Load data
    let data = load_scatter_data(
        tsv_reader,
        x_idx,
        &y_indices,
        &y_names,
        color_idx,
        ignore_errors,
    )?;

    if data.is_empty() || data.values().all(|v| v.is_empty()) {
        return Err(anyhow::anyhow!("No valid data points to plot"));
    }

    // Calculate bounds
    let all_points: Vec<(f64, f64)> = data.values().flatten().copied().collect();
    let (x_min, x_max, y_min, y_max) =
        axis::calculate_bounds(all_points.iter().copied());

    // Get chart dimensions
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

    // Build Y axis label
    let y_axis_label = if y_names.len() == 1 {
        y_names[0].clone()
    } else {
        format!("{} ({} series)", y_names.join(", "), y_names.len())
    };

    // Prepare regression data
    let mut regression_data: Vec<(String, Vec<(f64, f64)>, usize)> = Vec::new();
    if draw_regression {
        for (i, (group, points)) in data.iter().enumerate() {
            if let Some((slope, intercept)) = regression::calculate_regression(points) {
                let reg_points = regression::generate_regression_points(
                    slope, intercept, x_min, x_max, y_min, y_max,
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

    // Process data
    let datasets = process_scatter_data(data, is_line, regression_data);

    // Build config
    let mut config = ScatterConfig::new(chart_width, chart_height)
        .with_marker(marker)
        .with_labels(x_name, y_axis_label);

    if is_line {
        config = config.with_line();
    } else if is_path {
        config = config.with_path();
    }

    if draw_regression {
        config = config.with_regression();
    }

    // Render
    render_scatter_chart(datasets, x_min, x_max, y_min, y_max, &config)?;

    Ok(())
}
