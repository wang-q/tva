use crate::libs::tsv::fields::{parse_field_list_with_header_preserve_order, Header};
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::TsvRecord;
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
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line as a header"),
        )
        .arg(
            Arg::new("ignore-case")
                .short('i')
                .long("ignore-case")
                .help("Compare values case-insensitively")
                .action(ArgAction::SetTrue),
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
    let has_header = args.get_flag("header");

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

    let mut writer = crate::libs::io::writer(outfile);

    // State tracking: store previous values for selected columns
    // We only need to store previous values for the columns we are blanking
    let mut previous_values: HashMap<usize, String> = HashMap::new();
    let mut header_written = false;

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut reader = TsvReader::new(input.reader);
        let mut header_record = TsvRecord::new();
        let mut header: Option<Header> = None;

        // If header flag is set, read the first line as header
        if has_header {
            let mut has_record = false;
            // Only read one record
            reader
                .for_each_record(|rec| {
                    header_record.parse_line(rec, b'\t');
                    has_record = true;
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        "Stop iteration",
                    ))
                })
                .or_else(|e| {
                    if e.kind() == std::io::ErrorKind::Interrupted {
                        Ok(())
                    } else {
                        Err(e)
                    }
                })?;

            if !has_record {
                continue; // Empty file
            }

            // Write header only for the first file
            if !header_written {
                writer.write_all(header_record.as_line())?;
                writer.write_all(b"\n")?;
                header_written = true;
            }

            let header_str = std::str::from_utf8(header_record.as_line())?;
            header = Some(Header::from_line(header_str, '\t'));
        }

        // Map column indices to their replacement values
        // Using HashMap<usize, String> where key is 0-based column index
        let mut col_replacements: HashMap<usize, String> = HashMap::new();

        for config in &field_configs {
            // Parse the selector using tsv::fields logic
            // If no header, only numeric selectors are allowed
            let indices = parse_field_list_with_header_preserve_order(
                &config.selector,
                header.as_ref(),
                '\t',
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

            for idx in indices {
                // idx is 1-based, convert to 0-based
                col_replacements.insert(idx - 1, config.replacement.clone());
            }
        }

        reader.for_each_record(|record| {
            let mut current_col = 0;
            let mut last_pos = 0;
            let mut iter = memchr::memchr_iter(b'\t', record);

            loop {
                let (end_pos, is_last) = match iter.next() {
                    Some(pos) => (pos, false),
                    None => (record.len(), true),
                };

                if current_col > 0 {
                    writer.write_all(b"\t")?;
                }

                let cell_bytes = &record[last_pos..end_pos];
                let cell_str = std::str::from_utf8(cell_bytes).unwrap_or("");

                if let Some(replacement) = col_replacements.get(&current_col) {
                    // This column is subject to blanking
                    let should_blank =
                        if let Some(prev_val) = previous_values.get(&current_col) {
                            if ignore_case {
                                prev_val.eq_ignore_ascii_case(cell_str)
                            } else {
                                prev_val == cell_str
                            }
                        } else {
                            false // First row of data, never blank
                        };

                    if should_blank {
                        writer.write_all(replacement.as_bytes())?;
                    } else {
                        writer.write_all(cell_bytes)?;
                        // Update previous value
                        previous_values.insert(current_col, cell_str.to_string());
                    }
                } else {
                    // Not a blanking column, just write through
                    writer.write_all(cell_bytes)?;
                }

                if is_last {
                    break;
                }
                last_pos = end_pos + 1;
                current_col += 1;
            }
            writer.write_all(b"\n")?;
            Ok(())
        })?;
    }

    writer.flush()?;
    Ok(())
}
