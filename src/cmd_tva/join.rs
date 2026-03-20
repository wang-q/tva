use crate::libs::cli::{build_header_config, get_delimiter, header_args_with_columns};
use crate::libs::tsv::fields::FieldResolver;
use crate::libs::tsv::header::{write_header, Header};
use crate::libs::tsv::key::{KeyBuffer, KeyExtractor};
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::TsvRow;
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

/// Parse field specification for join keys using FieldResolver.
/// Returns (is_whole_line, field_indices).
/// Note: Key fields are sorted to ensure consistent key matching.
fn parse_join_field_spec(
    spec_opt: Option<&str>,
    resolver: &FieldResolver,
) -> anyhow::Result<(bool, Option<Vec<usize>>)> {
    let spec = spec_opt.unwrap_or("0");
    let trimmed = spec.trim();
    if trimmed == "0" {
        return Ok((true, None));
    }
    let mut indices = resolver
        .resolve(trimmed)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    // Sort and deduplicate key fields for consistent key matching
    indices.sort_unstable();
    indices.dedup();
    Ok((false, Some(indices)))
}

/// Parse field specification for append fields using FieldResolver.
fn parse_append_field_spec(
    spec_opt: Option<&str>,
    resolver: &FieldResolver,
) -> anyhow::Result<Option<Vec<usize>>> {
    let spec = match spec_opt {
        Some(s) => s,
        None => return Ok(None),
    };
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let indices = resolver
        .resolve(trimmed)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    if indices.is_empty() {
        Ok(None)
    } else {
        Ok(Some(indices))
    }
}

/// Extracts values to append from a TsvRow.
/// Values are stored as a single byte string with delimiters to avoid Vec<String> overhead.
fn extract_values(
    row: &TsvRow<'_, '_>,
    delimiter: u8,
    plan: &crate::libs::tsv::select::SelectPlan,
    ranges_buf: &mut Vec<Range<usize>>,
) -> anyhow::Result<Vec<u8>> {
    if let Err(idx) = plan.extract_ranges(row, ranges_buf) {
        let n = row.ends.len();
        anyhow::bail!(
            "line has {} fields, but append index {} is out of range",
            n,
            idx
        );
    }

    let mut values = Vec::with_capacity(row.line.len());
    let mut first = true;
    for range in ranges_buf.iter() {
        if !first {
            values.push(delimiter);
        }
        if range.start < range.end {
            values.extend_from_slice(&row.line[range.clone()]);
        }
        first = false;
    }
    Ok(values)
}

/// Build the append header suffix from filter header and append indices.
fn build_append_header_suffix(
    append_indices: Option<&Vec<usize>>,
    resolver: &FieldResolver,
    delimiter: char,
    prefix: &str,
) -> Option<String> {
    let idxs = append_indices?;
    let column_names = resolver.column_names()?;

    let mut s = String::new();
    for idx in idxs {
        let pos = idx - 1;
        // Note: Index out of range check is done in extract_values when processing data
        s.push(delimiter);
        if prefix.is_empty() {
            if pos < column_names.len() {
                s.push_str(&column_names[pos]);
            }
        } else {
            s.push_str(prefix);
            if pos < column_names.len() {
                s.push_str(&column_names[pos]);
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

    let opt_delimiter = get_delimiter(args, "delimiter")?;
    let delimiter_char = opt_delimiter as char;

    // Validate argument combinations
    if exclude && append_fields_spec.is_some() {
        anyhow::bail!("--exclude cannot be used with --append-fields");
    }
    if exclude && write_all_value.is_some() {
        anyhow::bail!("--write-all cannot be used with --exclude");
    }
    if write_all_value.is_some() && append_fields_spec.is_none() {
        anyhow::bail!("--write-all requires --append-fields");
    }
    if filter_file == "-" && infiles.len() == 1 && infiles[0] == "stdin" {
        anyhow::bail!("data file is required when filter-file is '-'");
    }

    let prefix = args
        .get_one::<String>("prefix")
        .map(|s| s.as_str())
        .unwrap_or("");
    if !header_config.enabled && !prefix.is_empty() {
        anyhow::bail!("--prefix requires --header");
    }

    // ============================================================
    // Phase 1: Process Filter File
    // ============================================================
    let mut filter_reader = TsvReader::new(crate::libs::io::raw_reader(&filter_file)?);

    // Read filter file header if enabled
    let mut filter_header_bytes: Option<Vec<u8>> = None;
    if header_config.enabled {
        if let Some(header_info) = filter_reader.read_header_mode(header_config.mode)? {
            filter_header_bytes = header_info.column_names_line.clone();
        }
    }

    // Create FieldResolver for filter file
    let filter_resolver =
        FieldResolver::new(filter_header_bytes.clone(), delimiter_char);

    // Parse filter file field specifications
    let (filter_key_whole_line, filter_key_indices) =
        parse_join_field_spec(key_fields_spec, &filter_resolver)?;
    let append_indices = parse_append_field_spec(append_fields_spec, &filter_resolver)?;
    let append_count = append_indices.as_ref().map(|v| v.len()).unwrap_or(0);

    // Build append header suffix for output
    let append_header_suffix = build_append_header_suffix(
        append_indices.as_ref(),
        &filter_resolver,
        delimiter_char,
        prefix,
    );
    let write_all_fill =
        build_write_all_fill(write_all_value, append_count, opt_delimiter);

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

    filter_reader.for_each_row(opt_delimiter, |row| {
        if row.line.is_empty() {
            return Ok(());
        }

        let key = match filter_key_extractor.extract_from_row(row, opt_delimiter) {
            Ok(k) => k,
            Err(idx) => {
                let n = row.ends.len();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "line has {} fields, but key index {} is out of range",
                        n, idx
                    ),
                ));
            }
        };

        let values = if let Some(ref plan) = append_plan {
            ranges_buf.clear();
            match extract_values(row, opt_delimiter, plan, &mut ranges_buf) {
                Ok(v) => v,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        e.to_string(),
                    ));
                }
            }
        } else {
            Vec::new()
        };

        if let Some(existing) = filter_map.get_mut(key.as_ref()) {
            if !allow_duplicate_keys && *existing != values {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "duplicate key with different append values found in filter file",
                ));
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
                    let data_resolver =
                        FieldResolver::new(Some(column_names.clone()), delimiter_char);
                    let effective_data_spec = data_fields_spec.or(key_fields_spec);

                    let (data_key_whole_line, indices) =
                        parse_join_field_spec(effective_data_spec, &data_resolver)?;

                    // Validate key lengths match
                    if !filter_key_whole_line && !data_key_whole_line {
                        let fk_len =
                            filter_key_indices.as_ref().map(|v| v.len()).unwrap_or(0);
                        let dk_len = indices.as_ref().map(|v| v.len()).unwrap_or(0);
                        if fk_len != dk_len {
                            anyhow::bail!(
                                "different number of key-fields and data-fields in file {}",
                                input.name
                            );
                        }
                    }

                    data_key_extractor = Some(KeyExtractor::new(indices, false, true));
                }

                // Write header only for the first file (only if there are column names)
                if !header_written {
                    if let Some(ref column_names) = header_info.column_names_line {
                        if !column_names.is_empty() {
                            // Convert to Header and write with optional suffix
                            let header = Header::from_info(header_info, delimiter_char);
                            let suffix =
                                append_header_suffix.as_deref().map(|s| s.as_bytes());
                            write_header(&mut writer, &header, suffix)?;
                            if line_buffered {
                                writer.flush()?;
                            }
                        }
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
            let data_resolver = FieldResolver::new(None, delimiter_char);
            let (data_key_whole_line, indices) =
                parse_join_field_spec(effective_data_spec, &data_resolver)?;

            // Validate key lengths match
            if !filter_key_whole_line && !data_key_whole_line {
                let fk_len = filter_key_indices.as_ref().map(|v| v.len()).unwrap_or(0);
                let dk_len = indices.as_ref().map(|v| v.len()).unwrap_or(0);
                if fk_len != dk_len {
                    anyhow::bail!("different number of key-fields and data-fields");
                }
            }

            data_key_extractor = Some(KeyExtractor::new(indices, false, true));
        }

        // Get a reference to the extractor (unwrap is safe here)
        let extractor = data_key_extractor
            .as_mut()
            .expect("data_key_extractor should be initialized");

        // Process data records
        reader.for_each_row(opt_delimiter, |row: &TsvRow| {
            if row.line.is_empty() {
                return Ok(());
            }

            let key = match extractor.extract_from_row(row, opt_delimiter) {
                Ok(k) => k,
                Err(idx) => {
                    let n = row.ends.len();
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "line has {} fields, but key index {} is out of range",
                            n, idx
                        ),
                    ));
                }
            };

            let matched = filter_map.get(key.as_ref());

            if exclude {
                if matched.is_none() {
                    writer.write_all(row.line)?;
                    writer.write_all(b"\n")?;
                    if line_buffered {
                        writer.flush()?;
                    }
                }
            } else if let Some(values) = matched {
                writer.write_all(row.line)?;
                if !values.is_empty() {
                    writer.write_all(&[opt_delimiter])?;
                    writer.write_all(values)?;
                }
                writer.write_all(b"\n")?;
                if line_buffered {
                    writer.flush()?;
                }
            } else if let Some(ref fill) = write_all_fill {
                writer.write_all(row.line)?;
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
