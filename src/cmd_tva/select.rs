use clap::*;
use std::collections::HashSet;
use std::ops::Range;

use crate::libs::cli::{build_header_config, header_args_with_columns};
use crate::libs::io::map_io_err;
use crate::libs::tsv::fields::FieldResolver;

use crate::libs::tsv::record::TsvRow;
use crate::libs::tsv::select;
use crate::libs::tsv::select::SelectPlan;

pub fn make_subcommand() -> Command {
    Command::new("select")
        .about("Selects and reorders TSV fields")
        .after_help(include_str!("../../docs/help/select.md"))
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
                .help("Field list to keep, using 1-based indices or names"),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .num_args(1)
                .help("Field list to drop from the output"),
        )
        .arg(
            Arg::new("rest")
                .long("rest")
                .short('r')
                .value_parser(["first", "last", "none"])
                .num_args(1)
                .help("Output location for fields not included in --fields"),
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
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

fn check_conflicts(selected: &[usize], excluded: &HashSet<usize>) -> anyhow::Result<()> {
    for idx in selected {
        if excluded.contains(idx) {
            anyhow::bail!("Field {} is both selected and excluded", idx);
        }
    }
    Ok(())
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let fields_spec: Option<String> = args.get_one::<String>("fields").cloned();
    let exclude_spec: Option<String> = args.get_one::<String>("exclude").cloned();

    if fields_spec.is_none() && exclude_spec.is_none() {
        anyhow::bail!("one of --fields/-f or --exclude/-e is required");
    }

    let rest_arg = args.get_one::<String>("rest").map(|s| s.as_str());
    let mut rest_mode = match rest_arg {
        Some("first") => select::RestMode::First,
        Some("last") => select::RestMode::Last,
        Some("none") => select::RestMode::None,
        _ => select::RestMode::None,
    };

    // Auto-detect rest mode if not specified
    if rest_arg.is_none() {
        if fields_spec.is_some() && exclude_spec.is_some() {
            rest_mode = select::RestMode::Last;
        } else if fields_spec.is_none() && exclude_spec.is_some() {
            rest_mode = select::RestMode::First;
        }
    }

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;
    let has_header = header_config.enabled;

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .cloned()
        .unwrap_or_else(|| "\t".to_string());
    let mut chars = delimiter_str.chars();
    let delimiter = chars.next().unwrap_or('\t');
    if chars.next().is_some() {
        anyhow::bail!(
            "delimiter must be a single character, got `{}`",
            delimiter_str
        );
    }
    let delim_byte = delimiter as u8;

    let mut header_written = false;
    let mut field_indices: Option<Vec<usize>> = None;
    let mut exclude_indices: Option<Vec<usize>> = None;
    let mut exclude_set: Option<HashSet<usize>> = None;

    let mut select_plan: Option<SelectPlan> = None;
    let mut output_ranges: Vec<Range<usize>> = Vec::new();

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut tsv_reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);

        if has_header {
            if !header_written {
                // First file: read header using the configured mode
                // header_config.mode defaults to FirstLine even when not explicitly enabled
                let header_result = tsv_reader
                    .read_header_mode(header_config.mode)
                    .map_err(map_io_err)?;

                // If no header found, skip this file (no output for empty input)
                let header_info = match header_result {
                    Some(info) => info,
                    None => continue,
                };

                // Get column names bytes
                // For HashLines1 mode, column_names_line must exist; otherwise skip
                let column_names_bytes: Vec<u8> = match header_info.column_names_line {
                    Some(line) => line,
                    None => {
                        // No column names line found (e.g., HashLines1 with only hash lines)
                        // Skip this file as we can't parse field names
                        continue;
                    }
                };

                // Create FieldResolver for field parsing
                let resolver =
                    FieldResolver::new(Some(column_names_bytes.clone()), delimiter);

                // Resolve fields if not yet resolved
                if field_indices.is_none() {
                    if let Some(ref spec) = fields_spec {
                        field_indices =
                            Some(resolver.resolve(spec).map_err(map_io_err)?);
                    }
                }

                // Resolve exclude if not yet resolved
                if exclude_indices.is_none() {
                    if let Some(ref spec) = exclude_spec {
                        let indices = resolver.resolve(spec).map_err(map_io_err)?;
                        exclude_set = Some(indices.iter().copied().collect());
                        exclude_indices = Some(indices);
                    }
                }

                // Check conflicts
                if let (Some(ref f), Some(ref e)) = (&field_indices, &exclude_set) {
                    check_conflicts(f, e)?;
                }

                let empty_vec = Vec::new();
                let f_indices = field_indices.as_ref().unwrap_or(&empty_vec);

                // Build TsvRow for header - need to compute field ends
                let mut header_ends = Vec::new();
                for pos in memchr::memchr_iter(delim_byte, &column_names_bytes) {
                    header_ends.push(pos);
                }
                header_ends.push(column_names_bytes.len());
                let header_row = TsvRow {
                    line: &column_names_bytes,
                    ends: &header_ends,
                };

                // Write output header (only column names line, not all header lines)
                select::write_with_rest(
                    &mut writer,
                    &header_row,
                    delim_byte,
                    f_indices,
                    exclude_set.as_ref(),
                    rest_mode,
                )
                .map_err(map_io_err)?;

                header_written = true;
            } else {
                // Subsequent files: skip their header using the same mode
                let _ = tsv_reader
                    .read_header_mode(header_config.mode)
                    .map_err(map_io_err)?;
            }
        }

        // Fallback resolution if not resolved by header (e.g. no header mode or first file was empty?)
        if fields_spec.is_some() && field_indices.is_none() {
            if let Some(ref spec) = fields_spec {
                // Check for field names without header
                if contains_field_names(spec) {
                    anyhow::bail!("field name requires header");
                }
                // Use FieldResolver without header for numeric-only specs
                let resolver = FieldResolver::new(None, delimiter);
                field_indices = Some(resolver.resolve(spec).map_err(map_io_err)?);
            }
        }

        if exclude_spec.is_some() && exclude_set.is_none() {
            if let Some(ref spec) = exclude_spec {
                // Check for field names without header
                if contains_field_names(spec) {
                    anyhow::bail!("field name requires header");
                }
                // Use FieldResolver without header for numeric-only specs
                let resolver = FieldResolver::new(None, delimiter);
                let indices = resolver.resolve(spec).map_err(map_io_err)?;
                exclude_set = Some(indices.iter().copied().collect());
                exclude_indices = Some(indices);
            }
        }

        // Ensure conflicts are checked at least once
        if let (Some(ref f), Some(ref e)) = (&field_indices, &exclude_set) {
            check_conflicts(f, e)?;
        }

        // Initialize Plan if possible
        if select_plan.is_none()
            && rest_mode == select::RestMode::None
            && exclude_set.is_none()
        {
            if let Some(ref idxs) = field_indices {
                select_plan = Some(SelectPlan::new(idxs));
            }
        }

        tsv_reader.for_each_row(delim_byte, |row| {
            if row.line.is_empty() {
                return Ok(());
            }

            if let Some(ref plan) = select_plan {
                select::write_selected_from_bytes(
                    &mut writer,
                    row,
                    delim_byte,
                    plan,
                    &mut output_ranges,
                )?;
            } else {
                let empty_vec = Vec::new();
                let f_indices = field_indices.as_ref().unwrap_or(&empty_vec);

                select::write_with_rest(
                    &mut writer,
                    row,
                    delim_byte,
                    f_indices,
                    exclude_set.as_ref(),
                    rest_mode,
                )?;
            }
            Ok(())
        })?;
    }

    Ok(())
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
            // Empty start or end is an incomplete range (numeric), not a field name
            // If either side is non-empty and not a valid usize, it's a field name pattern
            if (!start.is_empty() && start.parse::<usize>().is_err())
                || (!end.is_empty() && end.parse::<usize>().is_err())
            {
                return true;
            }
        } else if trimmed.parse::<usize>().is_err() {
            // Single token that is not a number - must be a field name
            return true;
        }
    }
    false
}
