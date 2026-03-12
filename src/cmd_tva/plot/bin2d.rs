use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::libs::io::reader;
use crate::libs::plot::bin2d::{render_bin2d_chart, Bin2dConfig};
use crate::libs::tsv::fields::{parse_field_list_with_header, Header};
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::{Row, TsvRecord};
use crate::libs::tsv::split::TsvSplitter;

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

    // Parse cols and rows
    let cols_str = matches.get_one::<String>("cols").unwrap();
    let rows_str = matches.get_one::<String>("rows").unwrap();
    let (term_width, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
    let width = crate::libs::plot::parse_chart_dimension(Some(cols_str), term_width, 80)?;
    let height = crate::libs::plot::parse_chart_dimension(Some(rows_str), term_height, 24)?;

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

    // Parse X column
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
    let x_idx = x_indices[0] - 1;

    // Parse Y column
    let y_indices =
        parse_field_list_with_header(y_col, header_for_parsing.as_ref(), '\t')
            .map_err(|e| anyhow::anyhow!("Invalid Y column spec: {}", e))?;
    if y_indices.is_empty() {
        return Err(anyhow::anyhow!("No valid Y column specified"));
    }
    if y_indices.len() > 1 {
        return Err(anyhow::anyhow!(
            "Y column must be a single column, got {} columns",
            y_indices.len()
        ));
    }
    let y_idx = y_indices[0] - 1;

    // Get column names
    let x_name = headers
        .get(x_idx)
        .map(|h| String::from_utf8_lossy(h).to_string())
        .unwrap_or_else(|| format!("col{}", x_idx + 1));
    let y_name = headers
        .get(y_idx)
        .map(|h| String::from_utf8_lossy(h).to_string())
        .unwrap_or_else(|| format!("col{}", y_idx + 1));

    // Collect data points
    let mut x_values: Vec<f64> = Vec::new();
    let mut y_values: Vec<f64> = Vec::new();
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

        // Parse Y value
        let y_bytes = match record.get_bytes(y_idx + 1) {
            Some(b) => b,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Column {} not found in line", y_col),
                ));
            }
        };

        let y_val = match parse_float(y_bytes) {
            Some(v) => v,
            None => {
                if ignore_errors {
                    return Ok(());
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot parse '{}' as number in column {}",
                        String::from_utf8_lossy(y_bytes),
                        y_col
                    ),
                ));
            }
        };

        x_values.push(x_val);
        y_values.push(y_val);

        Ok(())
    })?;

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

    // Build config
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

    // Render the heatmap
    render_bin2d_chart(&x_values, &y_values, &config)?;

    Ok(())
}

fn parse_float(bytes: &[u8]) -> Option<f64> {
    let s = std::str::from_utf8(bytes).ok()?;
    s.trim().parse::<f64>().ok()
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
