use crate::libs::cli::{build_header_config, get_delimiter, header_args_with_columns};
use crate::libs::io::map_io_err;
use crate::libs::tsv::fields::FieldResolver;
use crate::libs::tsv::header::{write_header, Header};
use crate::libs::tsv::key::KeyExtractor;
use crate::libs::tsv::reader::TsvReader;
use clap::*;
use rapidhash::rapidhash;
use std::collections::HashMap;
use std::io::Write;

/// Builds a KeyExtractor from field specification.
/// Uses FieldResolver for unified field parsing.
fn build_extractor(
    fields_spec: Option<&str>,
    column_names_bytes: Option<&[u8]>,
    delimiter: char,
    ignore_case: bool,
) -> anyhow::Result<KeyExtractor> {
    match fields_spec {
        None => {
            // Default: whole line
            Ok(KeyExtractor::new(None, ignore_case, false))
        }
        Some(spec) if spec.trim() == "0" => {
            // Special case: 0 means whole line
            Ok(KeyExtractor::new(None, ignore_case, false))
        }
        Some(spec) => {
            // Check if spec contains non-numeric tokens that would require header
            if column_names_bytes.is_none() && contains_field_names(spec) {
                anyhow::bail!("field name requires header");
            }

            // Use FieldResolver for field parsing
            let resolver =
                FieldResolver::new(column_names_bytes.map(|b| b.to_vec()), delimiter);
            let indices = resolver.resolve(spec).map_err(|e| anyhow::anyhow!(e))?;
            Ok(KeyExtractor::new(Some(indices), ignore_case, true))
        }
    }
}

/// Checks if a field specification contains field names (non-numeric tokens).
/// Returns true if the spec contains tokens that are not purely numeric/ranges.
fn contains_field_names(spec: &str) -> bool {
    for part in spec.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for range pattern (e.g., "1-3", "col1-col3")
        if let Some(dash_pos) = trimmed.find('-') {
            let start = &trimmed[..dash_pos];
            let end = &trimmed[dash_pos + 1..];
            // If either side is not a valid usize, it's a field name pattern
            if start.parse::<usize>().is_err() || end.parse::<usize>().is_err() {
                return true;
            }
        } else if trimmed.parse::<usize>().is_err() {
            // Single token that is not a number - must be a field name
            return true;
        }
    }
    false
}

pub fn make_subcommand() -> Command {
    Command::new("uniq")
        .about("Deduplicates TSV rows from one or more files")
        .after_help(include_str!("../../docs/help/uniq.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("fields")
                .long("fields")
                .short('f')
                .num_args(1)
                .help("TSV fields (1-based) to use as dedup key"),
        )
        .args(header_args_with_columns())
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .short('d')
                .num_args(1)
                .default_value("\t")
                .help("Field delimiter character (default: TAB)"),
        )
        .arg(
            Arg::new("ignore-case")
                .long("ignore-case")
                .short('i')
                .action(ArgAction::SetTrue)
                .help("Ignore case when comparing keys"),
        )
        .arg(
            Arg::new("repeated")
                .long("repeated")
                .short('r')
                .action(ArgAction::SetTrue)
                .help("Output only lines that are repeated based on the key"),
        )
        .arg(
            Arg::new("at-least")
                .long("at-least")
                .short('a')
                .num_args(1)
                .help("Output only lines that are repeated at least INT times"),
        )
        .arg(
            Arg::new("max")
                .long("max")
                .short('m')
                .num_args(1)
                .help("Max number of each unique key to output (zero is ignored)"),
        )
        .arg(
            Arg::new("equiv")
                .long("equiv")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("Append equivalence class IDs rather than only uniq entries"),
        )
        .arg(
            Arg::new("equiv-header")
                .long("equiv-header")
                .num_args(1)
                .help("Header name for the equivalence class ID field"),
        )
        .arg(
            Arg::new("equiv-start")
                .long("equiv-start")
                .num_args(1)
                .help("Starting value for equivalence class IDs"),
        )
        .arg(
            Arg::new("number")
                .long("number")
                .short('z')
                .action(ArgAction::SetTrue)
                .help("Append occurrence numbers for each key"),
        )
        .arg(
            Arg::new("number-header")
                .long("number-header")
                .num_args(1)
                .help("Header name for the occurrence number field"),
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

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let fields_spec: Option<String> = args.get_one::<String>("fields").cloned();

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    let opt_delimiter = get_delimiter(args, "delimiter")? as char;

    let ignore_case = args.get_flag("ignore-case");
    let repeated = args.get_flag("repeated");
    let equiv_mode = args.get_flag("equiv");
    let number_mode = args.get_flag("number");
    let line_buffered = args.get_flag("line-buffered");

    let mut at_least: u64 = args
        .get_one::<String>("at-least")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let mut max: u64 = args
        .get_one::<String>("max")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    // When --repeated is set, we want lines that appear at least 2 times
    if repeated && at_least <= 1 {
        at_least = 2;
    }
    // Ensure max is at least at_least when:
    // - at_least >= 2 (we're filtering for repeated lines)
    // - max is less than at_least (current max won't satisfy the constraint)
    // - Either max is explicitly set to 0 (no limit), OR we're not in equiv/number mode
    //   (in equiv/number mode, max=0 means output all occurrences, so we don't need to adjust)
    if at_least >= 2 && max < at_least && (max != 0 || (!equiv_mode && !number_mode)) {
        max = at_least;
    }

    if !equiv_mode {
        if args.get_one::<String>("equiv-header").is_some() {
            anyhow::bail!("--equiv-header requires --equiv");
        }
        if args.get_one::<String>("equiv-start").is_some() {
            anyhow::bail!("--equiv-start requires --equiv");
        }
    }

    if !number_mode && args.get_one::<String>("number-header").is_some() {
        anyhow::bail!("--number-header requires --number");
    }

    let equiv_header = args
        .get_one::<String>("equiv-header")
        .cloned()
        .unwrap_or_else(|| "equiv_id".to_string());
    let number_header = args
        .get_one::<String>("number-header")
        .cloned()
        .unwrap_or_else(|| "equiv_line".to_string());
    let equiv_start = args
        .get_one::<String>("equiv-start")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1);
    let mut next_equiv_id: u64 = if equiv_start < 0 {
        eprintln!(
            "tva uniq: warning: --equiv-start value {} is negative, using 1 instead",
            equiv_start
        );
        1
    } else {
        equiv_start as u64
    };

    let mut header_written = false;
    let mut column_names_bytes: Option<Vec<u8>> = None;

    // Extractor handles key extraction logic
    let mut extractor: Option<KeyExtractor> = None;

    struct EquivEntry {
        equiv_id: u64,
        count: u64,
    }

    let mut equiv_map: HashMap<u64, EquivEntry> = HashMap::new();

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut tsv_reader = TsvReader::with_capacity(input.reader, 512 * 1024);

        // If header is enabled, read header according to the configured mode
        if header_config.enabled {
            let header_result = tsv_reader
                .read_header_mode(header_config.mode)
                .map_err(map_io_err)?;

            if let Some(h_info) = header_result {
                // Store column names for field resolution
                if column_names_bytes.is_none() {
                    column_names_bytes = h_info.column_names_line.clone();
                }

                // Write header only for the first file
                if !header_written {
                    // Convert to Header and write with suffix for equiv/number mode
                    let header = Header::from_info(h_info, opt_delimiter);

                    // Build suffix for additional columns
                    let mut suffix_items: Vec<&str> = Vec::new();
                    if equiv_mode {
                        suffix_items.push(&equiv_header);
                    }
                    if number_mode {
                        suffix_items.push(&number_header);
                    }
                    let suffix = if suffix_items.is_empty() {
                        None
                    } else {
                        Some(
                            suffix_items
                                .iter()
                                .flat_map(|s| {
                                    let mut v = vec![opt_delimiter as u8];
                                    v.extend_from_slice(s.as_bytes());
                                    v
                                })
                                .collect::<Vec<u8>>(),
                        )
                    };

                    write_header(&mut writer, &header, suffix.as_deref())?;
                    if line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }
            } else {
                continue; // Empty file
            }
        }

        // Build extractor if not already done
        if extractor.is_none() {
            extractor = Some(build_extractor(
                fields_spec.as_deref(),
                column_names_bytes.as_deref(),
                opt_delimiter,
                ignore_case,
            )?);
        }

        tsv_reader
            .for_each_row(opt_delimiter as u8, |row| {
                let subject = {
                    let key_res = extractor
                        .as_mut()
                        .unwrap()
                        .extract_from_row(row, opt_delimiter as u8);

                    match key_res {
                        Ok(parsed_key) => rapidhash(parsed_key.as_ref()),
                        Err(missing_idx) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!(
                                    "Not enough fields in line. Missing field index: {}",
                                    missing_idx
                                ),
                            ));
                        }
                    }
                };

                let entry = equiv_map.entry(subject).or_insert_with(|| {
                    let id = next_equiv_id;
                    next_equiv_id += 1;
                    EquivEntry {
                        equiv_id: id,
                        count: 0,
                    }
                });
                entry.count += 1;

                let mut is_output = false;
                if entry.count == 1 {
                    if at_least <= 1 {
                        is_output = true;
                    }
                } else if (entry.count <= max && entry.count >= at_least)
                    || (equiv_mode && max == 0)
                    || (number_mode && max == 0)
                {
                    is_output = true;
                }

                if is_output {
                    writer.write_all(row.line)?;
                    if equiv_mode {
                        writer.write_all(&[opt_delimiter as u8])?;
                        writer.write_all(entry.equiv_id.to_string().as_bytes())?;
                    }
                    if number_mode {
                        writer.write_all(&[opt_delimiter as u8])?;
                        writer.write_all(entry.count.to_string().as_bytes())?;
                    }
                    writer.write_all(b"\n")?;

                    if line_buffered {
                        writer.flush()?;
                    }
                }
                Ok(())
            })
            .map_err(map_io_err)?;
    }

    Ok(())
}
