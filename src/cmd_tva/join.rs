use clap::*;
use std::collections::HashMap;
use std::io::BufRead;

pub fn make_subcommand() -> Command {
    Command::new("join")
        .about("Joins TSV data with a filter file by key fields")
        .after_help(
            r###"
Joins lines from a TSV data stream against a filter file using one or more
key fields.

Input:
- The filter file is specified with --filter-file and is read into memory.
- Data is read from files or standard input.
- Files ending in '.gz' are transparently decompressed.

Keys:
- --key-fields/-k selects key fields from the filter file.
  * Default: 0 (use entire line as the key).
- --data-fields/-d selects key fields from the data stream, if different
  from --key-fields.
- Field lists support numeric indices and, with --header, field names
  and ranges.

Output:
- Matching lines from the data stream are written to standard output.
- When --append-fields/-a is given, the selected fields from the filter file
  are appended to each matching data line.
- When --header is set, exactly one header line is written, with any
  appended filter fields added to the data header.

Field syntax:
- Field lists support 1-based indices, ranges (1-3,5-7), header names,
  name ranges (run-user_time), and wildcards (*_time).
- Run `tva --help-fields` for a full description shared across tva commands.
"###,
        )
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
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line of each file as a header; only the first header is output"),
        )
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

fn parse_join_field_spec(
    spec_opt: Option<String>,
    header: Option<&crate::libs::fields::Header>,
    delimiter: char,
) -> (bool, Option<Vec<usize>>) {
    let spec = spec_opt.unwrap_or_else(|| "0".to_string());
    let trimmed = spec.trim();
    if trimmed == "0" {
        return (true, None);
    }
    let indices =
        crate::libs::fields::parse_field_list_with_header(trimmed, header, delimiter)
            .unwrap_or_else(|e| arg_error(&e));
    (false, Some(indices))
}

fn parse_append_field_spec(
    spec_opt: Option<String>,
    header: Option<&crate::libs::fields::Header>,
    delimiter: char,
) -> Option<Vec<usize>> {
    let spec = match spec_opt {
        Some(s) => s,
        None => return None,
    };
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return None;
    }
    let indices = crate::libs::fields::parse_field_list_with_header_preserve_order(
        trimmed, header, delimiter,
    )
    .unwrap_or_else(|e| arg_error(&e));
    if indices.is_empty() {
        None
    } else {
        Some(indices)
    }
}

fn key_from_indices(line: &str, indices: &[usize], delimiter: char) -> String {
    let fields: Vec<&str> = line.split(delimiter).collect();
    let mut parts: Vec<&str> = Vec::new();
    for idx in indices {
        let pos = *idx - 1;
        if pos >= fields.len() {
            eprintln!(
                "tva join: line has {} fields, but key index {} is out of range",
                fields.len(),
                idx
            );
            std::process::exit(1);
        }
        parts.push(fields[pos]);
    }
    parts.join(&delimiter.to_string())
}

fn values_from_indices(line: &str, indices: &[usize], delimiter: char) -> Vec<String> {
    let fields: Vec<&str> = line.split(delimiter).collect();
    let mut values: Vec<String> = Vec::new();
    for idx in indices {
        let pos = *idx - 1;
        if pos >= fields.len() {
            eprintln!(
                "tva join: line has {} fields, but append index {} is out of range",
                fields.len(),
                idx
            );
            std::process::exit(1);
        }
        values.push(fields[pos].to_string());
    }
    values
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let filter_file = args.get_one::<String>("filter-file").unwrap().to_string();
    let key_fields_spec: Option<String> = args.get_one::<String>("key-fields").cloned();
    let data_fields_spec: Option<String> =
        args.get_one::<String>("data-fields").cloned();
    let append_fields_spec: Option<String> =
        args.get_one::<String>("append-fields").cloned();
    let write_all_value: Option<String> = args.get_one::<String>("write-all").cloned();

    let has_header = args.get_flag("header");
    let allow_duplicate_keys = args.get_flag("allow-duplicate-keys");
    let line_buffered = args.get_flag("line-buffered");
    let exclude = args.get_flag("exclude");

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .cloned()
        .unwrap_or_else(|| "\t".to_string());
    let mut chars = delimiter_str.chars();
    let delimiter = chars.next().unwrap_or('\t');
    if chars.next().is_some() {
        arg_error(&format!(
            "delimiter must be a single character, got `{}`",
            delimiter_str
        ));
    }

    if exclude && append_fields_spec.is_some() {
        arg_error("--exclude cannot be used with --append-fields");
    }

    if exclude && write_all_value.is_some() {
        arg_error("--write-all cannot be used with --exclude");
    }

    if write_all_value.is_some() && append_fields_spec.is_none() {
        arg_error("--write-all requires --append-fields");
    }

    if filter_file == "-" && (infiles.len() == 1 && infiles[0] == "stdin") {
        arg_error("data file is required when filter-file is '-'");
    }

    let filter_reader = crate::libs::io::reader(&filter_file);
    let mut filter_lines_iter = filter_reader.lines().map_while(Result::ok);

    let mut filter_header: Option<crate::libs::fields::Header> = None;
    if has_header {
        if let Some(mut header_line) = filter_lines_iter.next() {
            if let Some('\r') = header_line.chars().last() {
                header_line.pop();
            }
            filter_header = Some(crate::libs::fields::Header::from_line(
                &header_line,
                delimiter,
            ));
        }
    }

    let (filter_key_whole_line, filter_key_indices) = parse_join_field_spec(
        key_fields_spec.clone(),
        filter_header.as_ref(),
        delimiter,
    );
    let append_indices =
        parse_append_field_spec(append_fields_spec, filter_header.as_ref(), delimiter);
    let append_count = append_indices.as_ref().map(|v| v.len()).unwrap_or(0);

    let mut filter_map: HashMap<String, Vec<String>> = HashMap::new();

    for mut line in filter_lines_iter {
        if let Some('\r') = line.chars().last() {
            line.pop();
        }
        if line.is_empty() {
            continue;
        }
        let key = if filter_key_whole_line {
            line.clone()
        } else {
            key_from_indices(&line, filter_key_indices.as_ref().unwrap(), delimiter)
        };

        let values = match append_indices.as_ref() {
            Some(idxs) => values_from_indices(&line, idxs, delimiter),
            None => Vec::new(),
        };

        if let Some(existing) = filter_map.get_mut(&key) {
            if !allow_duplicate_keys && *existing != values {
                eprintln!(
                    "tva join: duplicate key with different append values found in filter file"
                );
                std::process::exit(1);
            }
            if allow_duplicate_keys {
                *existing = values;
            }
        } else {
            filter_map.insert(key, values);
        }
    }

    let mut header_written = false;
    let prefix = args
        .get_one::<String>("prefix")
        .cloned()
        .unwrap_or_default();

    if !has_header && !prefix.is_empty() {
        arg_error("--prefix requires --header");
    }

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;
        let mut is_first_line = true;
        let mut data_key_whole_line = false;
        let mut data_key_indices: Option<Vec<usize>> = None;

        for mut line in reader.lines().map_while(Result::ok) {
            if let Some('\r') = line.chars().last() {
                line.pop();
            }
            if line.is_empty() {
                continue;
            }

            if has_header && is_first_line {
                if !header_written {
                    let effective_data_spec =
                        data_fields_spec.clone().or_else(|| key_fields_spec.clone());
                    let (whole_line, indices) = parse_join_field_spec(
                        effective_data_spec,
                        Some(&crate::libs::fields::Header::from_line(&line, delimiter)),
                        delimiter,
                    );
                    data_key_whole_line = whole_line;
                    data_key_indices = indices;

                    if !filter_key_whole_line && !data_key_whole_line {
                        if let (Some(ref fk), Some(ref dk)) =
                            (filter_key_indices.as_ref(), data_key_indices.as_ref())
                        {
                            if fk.len() != dk.len() {
                                eprintln!(
                                    "tva join: different number of key-fields and data-fields"
                                );
                                std::process::exit(1);
                            }
                        }
                    }

                    let mut header_line = line.clone();
                    if let Some(idxs) = append_indices.as_ref() {
                        if let Some(ref fh) = filter_header {
                            for idx in idxs {
                                let pos = *idx - 1;
                                if pos >= fh.fields.len() {
                                    eprintln!(
                                        "tva join: append index {} is out of range for filter header",
                                        idx
                                    );
                                    std::process::exit(1);
                                }
                                header_line.push(delimiter);
                                if prefix.is_empty() {
                                    header_line.push_str(&fh.fields[pos]);
                                } else {
                                    header_line.push_str(&prefix);
                                    header_line.push_str(&fh.fields[pos]);
                                }
                            }
                        }
                    }
                    writer.write_fmt(format_args!("{}\n", header_line))?;
                    if line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }

                is_first_line = false;
                continue;
            }

            is_first_line = false;

            if data_key_indices.is_none() {
                let effective_data_spec =
                    data_fields_spec.clone().or_else(|| key_fields_spec.clone());
                let (whole_line, indices) =
                    parse_join_field_spec(effective_data_spec.clone(), None, delimiter);
                data_key_whole_line = whole_line;
                data_key_indices = indices;

                if !filter_key_whole_line && !data_key_whole_line {
                    if let (Some(ref fk), Some(ref dk)) =
                        (filter_key_indices.as_ref(), data_key_indices.as_ref())
                    {
                        if fk.len() != dk.len() {
                            eprintln!("tva join: different number of key-fields and data-fields");
                            std::process::exit(1);
                        }
                    }
                }
            }

            let key = if data_key_whole_line {
                line.clone()
            } else {
                key_from_indices(&line, data_key_indices.as_ref().unwrap(), delimiter)
            };

            let matched = filter_map.get(&key);

            if exclude {
                if matched.is_none() {
                    writer.write_fmt(format_args!("{}\n", line))?;
                    if line_buffered {
                        writer.flush()?;
                    }
                }
            } else if let Some(values) = matched {
                let mut out_line = line.clone();
                if !values.is_empty() {
                    for v in values {
                        out_line.push(delimiter);
                        out_line.push_str(v);
                    }
                }
                writer.write_fmt(format_args!("{}\n", out_line))?;
                if line_buffered {
                    writer.flush()?;
                }
            } else if let Some(ref fill) = write_all_value {
                let mut out_line = line.clone();
                if append_count > 0 {
                    for _ in 0..append_count {
                        out_line.push(delimiter);
                        out_line.push_str(fill);
                    }
                }
                writer.write_fmt(format_args!("{}\n", out_line))?;
                if line_buffered {
                    writer.flush()?;
                }
            }
        }
    }

    Ok(())
}
