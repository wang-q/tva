use crate::libs::cli::{build_header_config, header_args_with_columns};
use crate::libs::tsv::fields::{parse_field_list_with_header_preserve_order, Header};
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::split::TsvSplitter;
use clap::*;
use std::collections::HashMap;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("fill")
        .about("Fills missing values in selected columns")
        .after_help(include_str!("../../docs/help/fill.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("field")
                .short('f')
                .long("field")
                .help("Column to fill (format: COL)")
                .num_args(1)
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("value")
                .short('v')
                .long("value")
                .help("Constant value to fill with (if provided, overrides 'down' direction)")
                .num_args(1),
        )
        .arg(
            Arg::new("na")
                .long("na")
                .help("String to consider as missing/NA (default: empty string)")
                .num_args(1)
                .default_value(""),
        )
        .arg(
            Arg::new("direction")
                .long("direction")
                .help("Direction to fill: 'down' (default) or 'const' (implied by --value)")
                .value_parser(["down"])
                .default_value("down"),
        )
        .args(header_args_with_columns())
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .help("Output filename. [stdout] for screen")
                .num_args(1)
                .default_value("stdout"),
        )
        .arg(
            Arg::new("line-buffered")
                .long("line-buffered")
                .action(ArgAction::SetTrue)
                .help("Enable line-buffered output (flush after each line)"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };
    let outfile = args.get_one::<String>("outfile").unwrap();
    let line_buffered = args.get_flag("line-buffered");

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    let na_str = args
        .get_one::<String>("na")
        .map(|s| s.as_str())
        .unwrap_or("");
    let na_bytes = na_str.as_bytes();

    let const_value = args.get_one::<String>("value");

    // Parse field configurations
    let field_specs: Vec<String> = args
        .get_many::<String>("field")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let mut writer = crate::libs::io::writer(outfile)?;

    // State tracking: store last valid values for selected columns
    let mut last_valid_values: HashMap<usize, Vec<u8>> = HashMap::new();
    let mut header_written = false;

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut reader = TsvReader::new(input.reader);
        let mut header: Option<Header> = None;

        // If header is enabled, read header according to the configured mode
        if header_config.enabled {
            let header_result = reader
                .read_header_mode(header_config.mode)
                .map_err(|e| anyhow::anyhow!(e))?;

            if let Some(header_info) = header_result {
                // Write header only for the first file
                if !header_written {
                    // Write all header lines (hash lines, or LinesN lines)
                    for line in &header_info.lines {
                        writer.write_all(line)?;
                        writer.write_all(b"\n")?;
                    }
                    // For modes that provide column names, also write the column names line
                    if let Some(ref column_names) = header_info.column_names_line {
                        writer.write_all(column_names)?;
                        writer.write_all(b"\n")?;
                    }
                    if line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }

                // Get column names for field resolution if available
                if let Some(column_names_bytes) = header_info.column_names_line {
                    let header_str = std::str::from_utf8(&column_names_bytes)?;
                    header = Some(Header::from_line(header_str, '\t'));
                }
            } else {
                continue; // Empty file
            }
        }

        // Identify which columns need to be filled.
        let mut target_cols: Vec<usize> = Vec::new();

        for spec in &field_specs {
            let indices =
                parse_field_list_with_header_preserve_order(spec, header.as_ref(), '\t')
                    .map_err(|e| anyhow::anyhow!(e))?;

            for idx in indices {
                // idx is 1-based, convert to 0-based
                target_cols.push(idx - 1);
            }
        }
        // Deduplicate and sort for fast binary search during iteration
        target_cols.sort_unstable();
        target_cols.dedup();

        reader.for_each_record(|record| {
            let mut first = true;
            for (current_col, cell_bytes) in TsvSplitter::new(record, b'\t').enumerate()
            {
                if !first {
                    writer.write_all(b"\t")?;
                }
                first = false;

                // Check if this column is targeted for filling
                if target_cols.binary_search(&current_col).is_ok() {
                    let is_na = cell_bytes == na_bytes;

                    if is_na {
                        // Value is missing, try to fill it
                        if let Some(val) = const_value {
                            // Strategy: Constant fill
                            writer.write_all(val.as_bytes())?;
                        } else if let Some(prev) = last_valid_values.get(&current_col) {
                            // Strategy: Down fill (LOCF - Last Observation Carried Forward)
                            writer.write_all(prev)?;
                        } else {
                            // No previous value available and no constant provided
                            // Keep the original NA value
                            writer.write_all(cell_bytes)?;
                        }
                    } else {
                        // Value is valid (not NA)
                        // If using 'down' fill, update the last seen valid value for this column
                        if const_value.is_none() {
                            last_valid_values.insert(current_col, cell_bytes.to_vec());
                        }
                        writer.write_all(cell_bytes)?;
                    }
                } else {
                    // Not a target column, just write through
                    writer.write_all(cell_bytes)?;
                }
            }
            writer.write_all(b"\n")?;
            if line_buffered {
                writer.flush()?;
            }
            Ok(())
        })?;
    }

    writer.flush()?;
    Ok(())
}
