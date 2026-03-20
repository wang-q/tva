use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexMap;

use crate::libs::io::reader;
use crate::libs::plot::{
    boxplot::{render_boxplot, BoxPlotConfig},
    load_box_data, parse_chart_dimension, parse_columns, read_headers,
    stats::{calculate_bounds_from_stats, BoxStats},
};
use crate::libs::tsv::reader::TsvReader;

pub fn make_subcommand() -> Command {
    Command::new("box")
        .about("Draw a box plot (box-and-whisker plot)")
        .after_help(include_str!("../../../docs/help/plot_box.md"))
        .arg(
            Arg::new("y")
                .short('y')
                .long("y")
                .required(true)
                .help("Y axis column(s) to plot (1-based index or column name)"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .help("Color grouping column (1-based index or column name)"),
        )
        .arg(
            Arg::new("outliers")
                .long("outliers")
                .action(ArgAction::SetTrue)
                .help("Show outlier points"),
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
    let y_col = matches.get_one::<String>("y").unwrap();
    let color_col = matches.get_one::<String>("color");
    let show_outliers = matches.get_flag("outliers");
    let ignore_errors = matches.get_flag("ignore");

    // Open input
    let infile = matches.get_one::<String>("infile");
    let input_reader = match infile {
        Some(path) => reader(path)?,
        None => reader("stdin")?,
    };

    let mut tsv_reader: TsvReader<_> = TsvReader::new(input_reader);

    // Read headers
    let (headers, header_line) = read_headers(&mut tsv_reader)?;
    let header_line_ref = header_line.as_deref();

    // Parse Y columns
    let y_spec = parse_columns(y_col, header_line_ref, &headers)?;
    let y_indices = y_spec.indices;
    let y_names = y_spec.names;

    // Parse color column
    let color_idx = match color_col {
        Some(c) => {
            let (idx, _) =
                crate::libs::plot::parse_single_column(c, header_line_ref, &headers)?;
            Some(idx)
        }
        None => None,
    };

    // Load data
    let data =
        load_box_data(tsv_reader, &y_indices, &y_names, color_idx, ignore_errors)?;

    if data.is_empty() || data.values().all(|v| v.is_empty()) {
        return Err(anyhow::anyhow!("No valid data points to plot"));
    }

    // Calculate box statistics
    let mut box_data: IndexMap<String, BoxStats> = IndexMap::new();
    for (group, values) in data {
        if let Some(stats) = BoxStats::calculate(&values) {
            box_data.insert(group, stats);
        }
    }

    if box_data.is_empty() {
        return Err(anyhow::anyhow!("No valid statistics to plot"));
    }

    // Calculate Y bounds
    let (y_min, y_max) = calculate_bounds_from_stats(box_data.values());

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

    // Build config and render
    let config =
        BoxPlotConfig::new(chart_width, chart_height).with_outliers(show_outliers);

    render_boxplot(box_data, y_min, y_max, &config)?;

    Ok(())
}
