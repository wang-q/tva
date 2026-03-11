use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexMap;

use crate::libs::io::reader;
use crate::libs::plot::chart::{render_chart_box, ChartConfigBox};
use crate::libs::plot::stats::{calculate_bounds_from_stats, BoxStats as StatsBoxStats};
use crate::libs::tsv::fields::{parse_field_list_with_header, Header};
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::{Row, TsvRecord};
use crate::libs::tsv::split::TsvSplitter;

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

    // Parse Y columns
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

    // Get Y column names for labels
    let y_names: Vec<String> = y_indices
        .iter()
        .map(|&idx| {
            headers
                .get(idx)
                .map(|h| String::from_utf8_lossy(h).to_string())
                .unwrap_or_else(|| format!("col{}", idx + 1))
        })
        .collect();

    // Data storage: group_key -> Vec<values>
    let mut data: IndexMap<String, Vec<f64>> = IndexMap::new();
    let mut record = TsvRecord::new();

    tsv_reader.for_each_record(|line| {
        record.parse_line(line, b'\t');

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

            // Build group key
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

            data.entry(group_key).or_default().push(y_val);
        }

        Ok(())
    })?;

    if data.is_empty() || data.values().all(|v| v.is_empty()) {
        return Err(anyhow::anyhow!("No valid data points to plot"));
    }

    // Calculate box statistics for each group
    let mut box_data: IndexMap<String, StatsBoxStats> = IndexMap::new();
    for (group, values) in data {
        if let Some(stats) = StatsBoxStats::calculate(&values) {
            box_data.insert(group, stats);
        }
    }

    if box_data.is_empty() {
        return Err(anyhow::anyhow!("No valid statistics to plot"));
    }

    // Calculate Y bounds from all statistics
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

    // Build chart configuration
    let config =
        ChartConfigBox::new(chart_width, chart_height).with_outliers(show_outliers);

    // Render the box plot
    render_chart_box(box_data, y_min, y_max, &config)?;

    Ok(())
}

// calculate_bounds_from_stats is now in crate::libs::plot::stats

fn parse_chart_dimension(
    value: Option<&String>,
    term_size: u16,
    default: u16,
) -> Result<u16> {
    match value {
        None => Ok(default),
        Some(v) => {
            if v.contains('.') {
                let ratio: f64 = v.parse()?;
                let result = (term_size as f64 * ratio).round() as u16;
                Ok(result.max(10))
            } else {
                let result: u16 = v.parse()?;
                Ok(result.max(10))
            }
        }
    }
}

fn parse_float(bytes: &[u8]) -> Option<f64> {
    crate::libs::number::fast_parse_f64(bytes)
}
