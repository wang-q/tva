use crate::libs::cli::{build_header_config, header_args_with_columns};
use crate::libs::tsv::fields::FieldResolver;
use crate::libs::tsv::header::{write_header, Header};
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::{Row, TsvRow};
use clap::*;
use std::collections::HashMap;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("blank")
        .about("Replaces consecutive identical values in selected columns")
        .after_help(include_str!("../../docs/help/blank.md"))
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
                .help("Column to blank (format: COL or COL:REPLACEMENT)")
                .num_args(1)
                .action(ArgAction::Append)
                .required(true),
        )
        .args(header_args_with_columns())
        .arg(
            Arg::new("ignore-case")
                .short('i')
                .long("ignore-case")
                .help("Compare values case-insensitively")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("line-buffered")
                .long("line-buffered")
                .action(ArgAction::SetTrue)
                .help("Enable line-buffered output (flush after each line)"),
        )
        .arg(
            Arg::new("outfile")
                .short('o')
                .long("outfile")
                .help("Output filename. [stdout] for screen")
                .num_args(1)
                .default_value("stdout"),
        )
}

struct FieldConfig {
    selector: String,
    replacement: String,
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };
    let outfile = args.get_one::<String>("outfile").unwrap();
    let ignore_case = args.get_flag("ignore-case");
    let line_buffered = args.get_flag("line-buffered");

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    // Parse field configurations
    let field_specs: Vec<String> = args
        .get_many::<String>("field")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let mut field_configs = Vec::new();
    for spec in field_specs {
        let (selector, replacement) = if let Some(idx) = spec.find(':') {
            (spec[..idx].to_string(), spec[idx + 1..].to_string())
        } else {
            (spec, String::new())
        };
        field_configs.push(FieldConfig {
            selector,
            replacement,
        });
    }

    let mut writer = crate::libs::io::writer(outfile)?;

    // State tracking: store previous values for selected columns
    // We only need to store previous values for the columns we are blanking
    let mut previous_values: HashMap<usize, Vec<u8>> = HashMap::new();
    let mut header_written = false;

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut reader = TsvReader::new(input.reader);
        let mut column_names_bytes: Option<Vec<u8>> = None;

        // If header is enabled, read header according to the configured mode
        if header_config.enabled {
            let header_result = reader
                .read_header_mode(header_config.mode)
                .map_err(|e| anyhow::anyhow!(e))?;

            if let Some(header_info) = header_result {
                // Store column names for field resolution before moving header_info
                column_names_bytes = header_info.column_names_line.clone();

                // Write header only for the first file
                if !header_written {
                    let header = Header::from_info(header_info, '\t');
                    write_header(&mut writer, &header, None)?;
                    if line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }
            }
        }

        // Map column indices to their replacement values
        // Using HashMap<usize, Vec<u8>> where key is 0-based column index
        let mut col_replacements: HashMap<usize, Vec<u8>> = HashMap::new();

        // Create FieldResolver once for all field parsing
        let resolver = FieldResolver::new(column_names_bytes.clone(), '\t');

        for config in &field_configs {
            // Parse the selector using FieldResolver
            let indices = resolver
                .resolve(&config.selector)
                .map_err(|e| anyhow::anyhow!(e))?;

            for idx in indices {
                // idx is 1-based, convert to 0-based
                col_replacements.insert(idx - 1, config.replacement.as_bytes().to_vec());
            }
        }

        reader.for_each_row(b'\t', |row: &TsvRow| {
            let num_fields = row.field_count();
            for col_idx in 0..num_fields {
                if col_idx > 0 {
                    writer.write_all(b"\t")?;
                }

                let cell_bytes = row.get_bytes(col_idx + 1).unwrap_or(b"");

                if let Some(replacement) = col_replacements.get(&col_idx) {
                    // This column is subject to blanking
                    let should_blank =
                        if let Some(prev_val) = previous_values.get(&col_idx) {
                            if ignore_case {
                                prev_val.eq_ignore_ascii_case(cell_bytes)
                            } else {
                                prev_val.as_slice() == cell_bytes
                            }
                        } else {
                            false // First row of data, never blank
                        };

                    if should_blank {
                        writer.write_all(replacement)?;
                    } else {
                        writer.write_all(cell_bytes)?;
                        // Update previous value
                        previous_values.insert(col_idx, cell_bytes.to_vec());
                    }
                } else {
                    // Not a blanking column, just write through
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
