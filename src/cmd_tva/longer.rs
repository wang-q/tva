use crate::libs::cli::{build_header_config, header_args_with_columns};
use crate::libs::tsv::fields::FieldResolver;
use crate::libs::tsv::header::HeaderConfig;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::{Row, TsvRow};
use clap::*;
use regex::Regex;
use std::collections::HashSet;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("longer")
        .about("Reshapes a wide table into a long format")
        .after_help(include_str!("../../docs/help/longer.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("cols")
                .long("cols")
                .short('c')
                .required(true)
                .help("Columns to reshape"),
        )
        .arg(
            Arg::new("names-to")
                .long("names-to")
                .num_args(1..)
                .default_value("name")
                .help("Name of the new key column(s)"),
        )
        .arg(
            Arg::new("values-to")
                .long("values-to")
                .default_value("value")
                .help("Name of the new value column"),
        )
        .arg(
            Arg::new("names-pattern")
                .long("names-pattern")
                .help("Regex with capture groups to extract parts of column names"),
        )
        .arg(
            Arg::new("names-sep")
                .long("names-sep")
                .help("Separator to split column names into multiple columns"),
        )
        .arg(
            Arg::new("values-drop-na")
                .long("values-drop-na")
                .action(ArgAction::SetTrue)
                .help("Drop rows where the value is empty"),
        )
        .arg(
            Arg::new("names-prefix")
                .long("names-prefix")
                .help("A string to remove from the start of each variable name"),
        )
        .args(header_args_with_columns())
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

    let cols_spec = args.get_one::<String>("cols").unwrap();
    let names_to: Vec<&str> = args
        .get_many::<String>("names-to")
        .unwrap()
        .map(|s| s.as_str())
        .collect();
    let values_to = args.get_one::<String>("values-to").unwrap();
    let drop_na = args.get_flag("values-drop-na");
    let names_prefix = args.get_one::<String>("names-prefix").map(|s| s.as_str());
    let names_sep = args.get_one::<String>("names-sep").map(|s| s.as_str());
    let names_pattern = args
        .get_one::<String>("names-pattern")
        .map(|s| Regex::new(s))
        .transpose()?;

    if names_to.len() > 1 && names_sep.is_none() && names_pattern.is_none() {
        return Err(anyhow::anyhow!(
            "Multiple names-to provided but neither --names-sep nor --names-pattern is specified"
        ));
    }

    // Build HeaderConfig from arguments
    // longer defaults to header enabled (FirstLine mode) for backward compatibility
    let mut header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    // If no header mode is specified, default to FirstLine mode (backward compatibility)
    if !header_config.enabled {
        header_config = HeaderConfig::new().enabled().first_line();
    }

    let mut melt_indices: Option<Vec<usize>> = None;
    let mut id_indices: Option<Vec<usize>> = None;
    // Pre-computed bytes for the "name" column(s) for each melt index
    let mut melt_name_bytes: Option<Vec<Vec<u8>>> = None;
    let mut header_written = false;

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut reader = TsvReader::new(input.reader);
        let mut current_header_fields: Vec<String> = Vec::new();

        // Process header if enabled
        if header_config.enabled {
            let header_result = reader.read_header_mode(header_config.mode)?;

            if let Some(header_info) = header_result {
                // Parse column names from header
                if let Some(ref column_names) = header_info.column_names_line {
                    let header_str = std::str::from_utf8(column_names)?;
                    current_header_fields =
                        header_str.split('\t').map(|s| s.to_string()).collect();
                }

                // Mark header as written (for longer, we generate a new header)
                if !header_written {
                    header_written = true;
                }
            } else {
                continue; // Empty file
            }
        }

        // Initialize indices from the first file (only once)
        if melt_indices.is_none() {
            // Build header bytes from current_header_fields for FieldResolver
            let header_bytes = if current_header_fields.is_empty() {
                None
            } else {
                Some(current_header_fields.join("\t").into_bytes())
            };
            let resolver = FieldResolver::new(header_bytes, '\t');

            let melt_indices_1based = resolver
                .resolve(cols_spec)
                .map_err(|e| anyhow::anyhow!(e))?;

            // Check if any index is out of bounds
            let column_count = current_header_fields.len();
            if column_count == 0 {
                return Err(anyhow::anyhow!("Input file has no columns"));
            }
            for &idx in &melt_indices_1based {
                if idx == 0 || idx > column_count {
                    return Err(anyhow::anyhow!(
                        "Invalid column index {}: input file has only {} columns",
                        idx,
                        column_count
                    ));
                }
            }

            let melt_indices_local: Vec<usize> =
                melt_indices_1based.iter().map(|&i| i - 1).collect();
            let melt_set: HashSet<usize> = melt_indices_local.iter().cloned().collect();

            // Identify id columns (those not in melt_indices)
            let mut id_indices_local: Vec<usize> = Vec::new();
            for i in 0..current_header_fields.len() {
                if !melt_set.contains(&i) {
                    id_indices_local.push(i);
                }
            }

            // Write new header (only for the first file with header enabled)
            if header_config.enabled {
                let mut new_header_fields: Vec<String> =
                    Vec::with_capacity(id_indices_local.len() + names_to.len() + 1);
                for &i in &id_indices_local {
                    new_header_fields.push(current_header_fields[i].clone());
                }
                for name in &names_to {
                    new_header_fields.push(name.to_string());
                }
                new_header_fields.push(values_to.clone());
                writeln!(writer, "{}", new_header_fields.join("\t"))?;
            }

            // Pre-calculate name column bytes for each melt index
            let mut name_bytes_vec = Vec::with_capacity(melt_indices_local.len());
            for &idx in &melt_indices_local {
                let mut name_part = current_header_fields[idx].as_str();
                if let Some(prefix) = names_prefix {
                    if let Some(stripped) = name_part.strip_prefix(prefix) {
                        name_part = stripped;
                    }
                }

                let mut computed_name = String::new();
                if let Some(ref regex) = names_pattern {
                    if let Some(caps) = regex.captures(name_part) {
                        for i in 1..=names_to.len() {
                            if i > 1 {
                                computed_name.push('\t');
                            }
                            if let Some(m) = caps.get(i) {
                                computed_name.push_str(m.as_str());
                            }
                        }
                    } else {
                        // Fallback: write original name in first col, tabs for others
                        computed_name.push_str(name_part);
                        for _ in 1..names_to.len() {
                            computed_name.push('\t');
                        }
                    }
                } else if let Some(sep) = names_sep {
                    let parts: Vec<&str> = name_part.split(sep).collect();
                    for i in 0..names_to.len() {
                        if i > 0 {
                            computed_name.push('\t');
                        }
                        if i < parts.len() {
                            computed_name.push_str(parts[i]);
                        }
                    }
                } else {
                    computed_name.push_str(name_part);
                }
                name_bytes_vec.push(computed_name.into_bytes());
            }

            melt_indices = Some(melt_indices_local);
            id_indices = Some(id_indices_local);
            melt_name_bytes = Some(name_bytes_vec);
        }

        // Skip data processing if we haven't initialized indices (shouldn't happen)
        let melt_indices_ref = match melt_indices.as_ref() {
            Some(v) => v,
            None => continue,
        };
        let id_indices_ref = id_indices.as_ref().unwrap();
        let name_bytes = melt_name_bytes.as_ref().unwrap();

        // Process remaining rows
        reader.for_each_row(b'\t', |row: &TsvRow| {
            if row.line.is_empty() {
                return Ok(());
            }

            // Helper to get field bytes using TsvRow
            let get_field =
                |idx: usize| -> &[u8] { row.get_bytes(idx + 1).unwrap_or(b"") };

            // Iterate over melt columns
            for (k, &melt_idx) in melt_indices_ref.iter().enumerate() {
                let value = get_field(melt_idx);

                if drop_na && value.is_empty() {
                    continue;
                }

                // Write ID columns
                for (j, &id_idx) in id_indices_ref.iter().enumerate() {
                    if j > 0 {
                        writer.write_all(b"\t")?;
                    }
                    writer.write_all(get_field(id_idx))?;
                }

                // If we wrote ID columns, add tab separator
                if !id_indices_ref.is_empty() {
                    writer.write_all(b"\t")?;
                }

                // Write name column(s) (pre-calculated)
                writer.write_all(&name_bytes[k])?;

                // Write value
                writer.write_all(b"\t")?;
                writer.write_all(value)?;
                writer.write_all(b"\n")?;
            }

            Ok(())
        })?;
    }

    Ok(())
}
