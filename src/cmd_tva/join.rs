use crate::libs::cli::{build_header_config, header_args_with_columns};
use crate::libs::tsv::fields::{self, Header};
use crate::libs::tsv::key::{KeyBuffer, KeyExtractor};
use crate::libs::tsv::reader::TsvReader;
use ahash::RandomState;
use clap::*;
use std::collections::HashMap;
use std::io::Write;
use std::ops::Range;

pub fn make_subcommand() -> Command {
    Command::new("join")
        .about("Joins TSV data with a filter file by key fields")
        .after_help(include_str!("../../docs/help/join.md"))
        .arg(
            Arg::new("filter-file")
                .long("filter-file")
                .short('f')
                .num_args(1)
                .required(true)
                .help("Filter TSV file containing join keys"),
        )
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV data file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("key-fields")
                .long("key-fields")
                .short('k')
                .num_args(1)
                .help("Filter file fields to use as join key (default: 0 = entire line)"),
        )
        .arg(
            Arg::new("data-fields")
                .long("data-fields")
                .short('d')
                .num_args(1)
                .help("Data stream fields to use as join key (default: --key-fields)"),
        )
        .arg(
            Arg::new("append-fields")
                .long("append-fields")
                .short('a')
                .num_args(1)
                .help("Filter file fields to append to matched records"),
        )
        .arg(
            Arg::new("write-all")
                .long("write-all")
                .short('w')
                .num_args(1)
                .allow_hyphen_values(true)
                .help("Output all data records; use the given value for unmatched append fields"),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("Exclude matching records (anti-join)"),
        )
        .args(header_args_with_columns())
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .num_args(1)
                .default_value("\t")
                .help("Field delimiter character (default: TAB)"),
        )
        .arg(
            Arg::new("prefix")
                .long("prefix")
                .short('p')
                .num_args(1)
                .default_value("")
                .help("Prefix for appended header fields"),
        )
        .arg(
            Arg::new("allow-duplicate-keys")
                .long("allow-duplicate-keys")
                .short('z')
                .action(ArgAction::SetTrue)
                .help("Allow duplicate keys in the filter file (last entry wins)"),
        )
        .arg(
            Arg::new("line-buffered")
                .long("line-buffered")
                .action(ArgAction::SetTrue)
                .help("Enable line-buffered output (flush after each line)"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

fn arg_error(msg: &str) -> ! {
    eprintln!("tva join: {}", msg);
    std::process::exit(1);
}

/// Parse field specification for join keys.
/// Returns (is_whole_line, field_indices).
fn parse_join_field_spec(
    spec_opt: Option<&str>,
    header: Option<&Header>,
    delimiter: char,
) -> (bool, Option<Vec<usize>>) {
    let spec = spec_opt.unwrap_or("0");
    let trimmed = spec.trim();
    if trimmed == "0" {
        return (true, None);
    }
    let indices = fields::parse_field_list_with_header(trimmed, header, delimiter)
        .unwrap_or_else(|e| arg_error(&e));
    (false, Some(indices))
}

/// Parse field specification for append fields.
fn parse_append_field_spec(
    spec_opt: Option<&str>,
    header: Option<&Header>,
    delimiter: char,
) -> Option<Vec<usize>> {
    let spec = spec_opt?;
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return None;
    }
    let indices =
        fields::parse_field_list_with_header_preserve_order(trimmed, header, delimiter)
            .unwrap_or_else(|e| arg_error(&e));
    if indices.is_empty() {
        None
    } else {
        Some(indices)
    }
}

/// Count fields in a line.
fn count_fields(line: &[u8], delimiter: u8) -> usize {
    if line.is_empty() {
        0
    } else {
        line.iter().filter(|&&b| b == delimiter).count() + 1
    }
}

/// Extracts values to append from a line.
/// Values are stored as a single byte string with delimiters to avoid Vec<String> overhead.
fn extract_values(
    line: &[u8],
    delimiter: u8,
    plan: &crate::libs::tsv::select::SelectPlan,
    ranges_buf: &mut Vec<Range<usize>>,
) -> Vec<u8> {
    if let Err(idx) = plan.extract_ranges(line, delimiter, ranges_buf) {
        let n = count_fields(line, delimiter);
        eprintln!(
            "tva join: line has {} fields, but append index {} is out of range",
            n, idx
        );
        std::process::exit(1);
    }

    let mut values = Vec::with_capacity(line.len());
    let mut first = true;
    for range in ranges_buf.iter() {
        if !first {
            values.push(delimiter);
        }
        if range.start < range.end {
            values.extend_from_slice(&line[range.clone()]);
        }
        first = false;
    }
    values
}

/// Build the append header suffix from filter header and append indices.
fn build_append_header_suffix(
    append_indices: Option<&Vec<usize>>,
    filter_header: Option<&Header>,
    delimiter: char,
    prefix: &str,
) -> Option<String> {
    let idxs = append_indices?;
    let fh = filter_header?;

    let mut s = String::new();
    for idx in idxs {
        let pos = idx - 1;
        // Note: Index out of range check is done in extract_values when processing data
        s.push(delimiter);
        if prefix.is_empty() {
            if pos < fh.fields.len() {
                s.push_str(&fh.fields[pos]);
            }
        } else {
            s.push_str(prefix);
            if pos < fh.fields.len() {
                s.push_str(&fh.fields[pos]);
            }
        }
    }
    Some(s)
}

/// Build the fill value for unmatched records in write-all mode.
fn build_write_all_fill(
    write_all_value: Option<&str>,
    append_count: usize,
    delimiter: u8,
) -> Option<Vec<u8>> {
    let fill = write_all_value?;
    if append_count == 0 {
        return Some(Vec::new());
    }
    let mut s = Vec::with_capacity(append_count * (fill.len() + 1));
    for _ in 0..append_count {
        s.push(delimiter);
        s.extend_from_slice(fill.as_bytes());
    }
    Some(s)
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let filter_file = args.get_one::<String>("filter-file").unwrap().to_string();
    let key_fields_spec = args.get_one::<String>("key-fields").map(|s| s.as_str());
    let data_fields_spec = args.get_one::<String>("data-fields").map(|s| s.as_str());
    let append_fields_spec = args.get_one::<String>("append-fields").map(|s| s.as_str());
    let write_all_value = args.get_one::<String>("write-all").map(|s| s.as_str());

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    let allow_duplicate_keys = args.get_flag("allow-duplicate-keys");
    let line_buffered = args.get_flag("line-buffered");
    let exclude = args.get_flag("exclude");

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .map(|s| s.as_str())
        .unwrap_or("\t");
    let mut chars = delimiter_str.chars();
    let delimiter_char = chars.next().unwrap_or('\t');
    let delimiter = delimiter_char as u8;
    if chars.next().is_some() || delimiter_str.len() > 1 {
        arg_error(&format!(
            "delimiter must be a single character, got `{}`",
            delimiter_str
        ));
    }

    // Validate argument combinations
    if exclude && append_fields_spec.is_some() {
        arg_error("--exclude cannot be used with --append-fields");
    }
    if exclude && write_all_value.is_some() {
        arg_error("--write-all cannot be used with --exclude");
    }
    if write_all_value.is_some() && append_fields_spec.is_none() {
        arg_error("--write-all requires --append-fields");
    }
    if filter_file == "-" && infiles.len() == 1 && infiles[0] == "stdin" {
        arg_error("data file is required when filter-file is '-'");
    }

    let prefix = args
        .get_one::<String>("prefix")
        .map(|s| s.as_str())
        .unwrap_or("");
    if !header_config.enabled && !prefix.is_empty() {
        arg_error("--prefix requires --header");
    }

    // ============================================================
    // Phase 1: Process Filter File
    // ============================================================
    let mut filter_reader = TsvReader::new(crate::libs::io::raw_reader(&filter_file)?);

    // Read filter file header if enabled
    let mut filter_header: Option<Header> = None;
    if header_config.enabled {
        if let Some(header_info) = filter_reader.read_header_mode(header_config.mode)? {
            if let Some(ref column_names) = header_info.column_names_line {
                let header_str = std::str::from_utf8(column_names)?;
                filter_header = Some(Header::from_line(header_str, delimiter_char));
            }
        }
    }

    // Parse filter file field specifications
    let (filter_key_whole_line, filter_key_indices) =
        parse_join_field_spec(key_fields_spec, filter_header.as_ref(), delimiter_char);
    let append_indices = parse_append_field_spec(
        append_fields_spec,
        filter_header.as_ref(),
        delimiter_char,
    );
    let append_count = append_indices.as_ref().map(|v| v.len()).unwrap_or(0);

    // Build append header suffix for output
    let append_header_suffix = build_append_header_suffix(
        append_indices.as_ref(),
        filter_header.as_ref(),
        delimiter_char,
        prefix,
    );
    let write_all_fill = build_write_all_fill(write_all_value, append_count, delimiter);

    // Initialize key extractor and append plan
    let mut filter_key_extractor =
        KeyExtractor::new(filter_key_indices.clone(), false, true);
    let append_plan = append_indices
        .as_ref()
        .map(|idxs| crate::libs::tsv::select::SelectPlan::new(idxs));

    // Build filter hash map: Key -> Appended Values
    let mut filter_map: HashMap<KeyBuffer, Vec<u8>, RandomState> =
        HashMap::with_hasher(RandomState::new());
    let mut ranges_buf: Vec<Range<usize>> = Vec::new();

    filter_reader.for_each_record(|line| {
        if line.is_empty() {
            return Ok(());
        }

        let key = match filter_key_extractor.extract(line, delimiter) {
            Ok(k) => k,
            Err(idx) => {
                let n = count_fields(line, delimiter);
                eprintln!(
                    "tva join: line has {} fields, but key index {} is out of range",
                    n, idx
                );
                std::process::exit(1);
            }
        };

        let values = if let Some(ref plan) = append_plan {
            ranges_buf.clear();
            extract_values(line, delimiter, plan, &mut ranges_buf)
        } else {
            Vec::new()
        };

        if let Some(existing) = filter_map.get_mut(key.as_ref()) {
            if !allow_duplicate_keys && *existing != values {
                eprintln!("tva join: duplicate key with different append values found in filter file");
                std::process::exit(1);
            }
            if allow_duplicate_keys {
                *existing = values;
            }
        } else {
            filter_map.insert(key.into_owned(), values);
        }
        Ok(())
    })?;

    // ============================================================
    // Phase 2: Process Data Files
    // ============================================================
    let mut header_written = false;
    let mut data_key_extractor: Option<KeyExtractor> = None;

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut reader = TsvReader::with_capacity(input.reader, 512 * 1024);

        // Process header if enabled
        if header_config.enabled {
            let header_result = reader.read_header_mode(header_config.mode)?;

            if let Some(header_info) = header_result {
                // Parse data file header for field resolution (for each file, to handle different column orders)
                if let Some(ref column_names) = header_info.column_names_line {
                    let header_str = std::str::from_utf8(column_names)?;
                    let data_header = Header::from_line(header_str, delimiter_char);
                    let effective_data_spec = data_fields_spec.or(key_fields_spec);

                    let (data_key_whole_line, indices) = parse_join_field_spec(
                        effective_data_spec,
                        Some(&data_header),
                        delimiter_char,
                    );

                    // Validate key lengths match
                    if !filter_key_whole_line && !data_key_whole_line {
                        let fk_len =
                            filter_key_indices.as_ref().map(|v| v.len()).unwrap_or(0);
                        let dk_len = indices.as_ref().map(|v| v.len()).unwrap_or(0);
                        if fk_len != dk_len {
                            eprintln!(
                                "tva join: different number of key-fields and data-fields in file {}",
                                input.name
                            );
                            std::process::exit(1);
                        }
                    }

                    data_key_extractor = Some(KeyExtractor::new(indices, false, true));
                }

                // Write header only for the first file
                if !header_written {
                    // Write all header lines (hash lines, or LinesN lines)
                    for line in &header_info.lines {
                        writer.write_all(line)?;
                        writer.write_all(b"\n")?;
                    }
                    // Write column names line with append suffix
                    if let Some(ref column_names) = header_info.column_names_line {
                        if !column_names.is_empty() {
                            writer.write_all(column_names)?;
                            if let Some(ref suffix) = append_header_suffix {
                                writer.write_all(suffix.as_bytes())?;
                            }
                            writer.write_all(b"\n")?;
                        }
                    }
                    if line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }
            } else {
                continue; // Empty file
            }
        }

        // Initialize extractor for headerless files if not set
        if data_key_extractor.is_none() {
            let effective_data_spec = data_fields_spec.or(key_fields_spec);
            let (data_key_whole_line, indices) =
                parse_join_field_spec(effective_data_spec, None, delimiter_char);

            // Validate key lengths match
            if !filter_key_whole_line && !data_key_whole_line {
                let fk_len = filter_key_indices.as_ref().map(|v| v.len()).unwrap_or(0);
                let dk_len = indices.as_ref().map(|v| v.len()).unwrap_or(0);
                if fk_len != dk_len {
                    eprintln!(
                        "tva join: different number of key-fields and data-fields"
                    );
                    std::process::exit(1);
                }
            }

            data_key_extractor = Some(KeyExtractor::new(indices, false, true));
        }

        // Get a reference to the extractor (unwrap is safe here)
        let extractor = data_key_extractor
            .as_mut()
            .expect("data_key_extractor should be initialized");

        // Process data records
        reader.for_each_record(|line| {
            if line.is_empty() {
                return Ok(());
            }

            let key = match extractor.extract(line, delimiter) {
                Ok(k) => k,
                Err(idx) => {
                    let n = count_fields(line, delimiter);
                    eprintln!(
                        "tva join: line has {} fields, but key index {} is out of range",
                        n, idx
                    );
                    std::process::exit(1);
                }
            };

            let matched = filter_map.get(key.as_ref());

            if exclude {
                if matched.is_none() {
                    writer.write_all(line)?;
                    writer.write_all(b"\n")?;
                    if line_buffered {
                        writer.flush()?;
                    }
                }
            } else if let Some(values) = matched {
                writer.write_all(line)?;
                if !values.is_empty() {
                    writer.write_all(&[delimiter])?;
                    writer.write_all(values)?;
                }
                writer.write_all(b"\n")?;
                if line_buffered {
                    writer.flush()?;
                }
            } else if let Some(ref fill) = write_all_fill {
                writer.write_all(line)?;
                writer.write_all(fill)?;
                writer.write_all(b"\n")?;
                if line_buffered {
                    writer.flush()?;
                }
            }

            Ok(())
        })?;
    }

    Ok(())
}
