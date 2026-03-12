use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::libs::io::reader;
use crate::libs::plot::{
    binning::Bin2dConfig, build_header, heatmap::render_heatmap, load_bin2d_data,
    parse_chart_dimension, parse_single_column, read_headers,
};
use crate::libs::tsv::reader::TsvReader;

pub fn make_subcommand() -> Command {
    Command::new("bin2d")
        .about("Draw a 2D binning heatmap")
        .after_help(include_str!("../../../docs/help/plot_bin2d.md"))
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
            Arg::new("bins")
                .short('b')
                .long("bins")
                .default_value("30")
                .help(
                    "Number of bins in each direction (or 'x,y' for different counts)",
                ),
        )
        .arg(
            Arg::new("binwidth")
                .long("binwidth")
                .help("Width of bins (or 'x,y' for different widths)"),
        )
        .arg(
            Arg::new("strategy").short('S').long("strategy").help(
                "Strategy for automatic bin count: freedman-diaconis, sqrt, sturges",
            ),
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
    let x_col = matches.get_one::<String>("x").unwrap();
    let y_col = matches.get_one::<String>("y").unwrap();
    let bins_str = matches.get_one::<String>("bins").unwrap();
    let binwidth_str = matches.get_one::<String>("binwidth");
    let _strategy = matches.get_one::<String>("strategy");
    let ignore_errors = matches.get_flag("ignore");

    // Parse dimensions
    let (term_width, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
    let available_height = term_height.saturating_sub(1).max(10);
    let width =
        parse_chart_dimension(matches.get_one::<String>("cols"), term_width, 80)?;
    let height =
        parse_chart_dimension(matches.get_one::<String>("rows"), available_height, 24)?;

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

    // Parse columns
    let (x_idx, x_name) =
        parse_single_column(x_col, header_for_parsing.as_ref(), &headers)?;
    let (y_idx, y_name) =
        parse_single_column(y_col, header_for_parsing.as_ref(), &headers)?;

    // Load data
    let (x_values, y_values) =
        load_bin2d_data(tsv_reader, x_idx, &x_name, y_idx, &y_name, ignore_errors)?;

    if x_values.is_empty() {
        return Err(anyhow::anyhow!("No valid data points to plot"));
    }

    // Parse bins configuration
    let (x_bins, y_bins) = parse_bins_config(bins_str)?;

    // Parse binwidth configuration
    let (x_binwidth, y_binwidth) = match binwidth_str {
        Some(s) => parse_binwidth_config(s)?,
        None => (None, None),
    };

    // Build config and render
    let config = Bin2dConfig {
        width,
        height,
        x_bins,
        y_bins,
        x_binwidth,
        y_binwidth,
        x_label: x_name,
        y_label: y_name,
    };

    render_heatmap(&x_values, &y_values, &config)?;

    Ok(())
}

fn parse_bins_config(s: &str) -> Result<(usize, usize)> {
    if s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid bins format. Use 'N' or 'x_bins,y_bins'"
            ));
        }
        let x = parts[0].trim().parse::<usize>()?;
        let y = parts[1].trim().parse::<usize>()?;
        Ok((x, y))
    } else {
        let n = s.parse::<usize>()?;
        Ok((n, n))
    }
}

fn parse_binwidth_config(s: &str) -> Result<(Option<f64>, Option<f64>)> {
    if s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid binwidth format. Use 'W' or 'x_width,y_width'"
            ));
        }
        let x = parts[0].trim().parse::<f64>()?;
        let y = parts[1].trim().parse::<f64>()?;
        Ok((Some(x), Some(y)))
    } else {
        let w = s.parse::<f64>()?;
        Ok((Some(w), Some(w)))
    }
}
