use clap::*;
use std::collections::HashMap;
use std::io::Write;
use std::ops::Range;
use smallvec::SmallVec;
use ahash::RandomState;

type KeyBuffer = SmallVec<[u8; 32]>;

enum JoinKey<'a> {
    Ref(&'a [u8]),
    Owned(KeyBuffer),
}

impl<'a> AsRef<[u8]> for JoinKey<'a> {
    fn as_ref(&self) -> &[u8] {
        match self {
            JoinKey::Ref(s) => s,
            JoinKey::Owned(s) => s.as_ref(),
        }
    }
}

impl<'a> JoinKey<'a> {
    fn into_owned(self) -> KeyBuffer {
        match self {
            JoinKey::Ref(s) => KeyBuffer::from_slice(s),
            JoinKey::Owned(s) => s,
        }
    }
}

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
    header: Option<&crate::libs::tsv::fields::Header>,
    delimiter: char,
) -> (bool, Option<Vec<usize>>) {
    let spec = spec_opt.unwrap_or_else(|| "0".to_string());
    let trimmed = spec.trim();
    if trimmed == "0" {
        return (true, None);
    }
    let indices = crate::libs::tsv::fields::parse_field_list_with_header(
        trimmed, header, delimiter,
    )
    .unwrap_or_else(|e| arg_error(&e));
    (false, Some(indices))
}

fn parse_append_field_spec(
    spec_opt: Option<String>,
    header: Option<&crate::libs::tsv::fields::Header>,
    delimiter: char,
) -> Option<Vec<usize>> {
    let spec = spec_opt?;
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return None;
    }
    let indices = crate::libs::tsv::fields::parse_field_list_with_header_preserve_order(
        trimmed, header, delimiter,
    )
    .unwrap_or_else(|e| arg_error(&e));
    if indices.is_empty() {
        None
    } else {
        Some(indices)
    }
}

// Extracts key from line using ranges. Zero-allocation for key extraction if we had a proper buffer.
// Here we still need to return a Vec<u8> as the key to store in HashMap.
// But we can construct it efficiently.
fn extract_key<'a>(
    line: &'a [u8],
    delimiter: u8,
    whole_line: bool,
    plan: Option<&crate::libs::select::SelectPlan>,
    ranges_buf: &mut Vec<Range<usize>>,
) -> JoinKey<'a> {
    if whole_line {
        return JoinKey::Ref(line);
    }
    let plan = plan.unwrap();
    if let Err(idx) = plan.extract_ranges(line, delimiter, ranges_buf) {
        let n = if line.is_empty() {
            0
        } else {
            line.iter().filter(|&&b| b == delimiter).count() + 1
        };
        eprintln!(
            "tva join: line has {} fields, but key index {} is out of range",
            n, idx
        );
        std::process::exit(1);
    }
    
    // Optimization: if single key field, return slice directly
    if ranges_buf.len() == 1 {
        return JoinKey::Ref(&line[ranges_buf[0].clone()]);
    }

    // Construct key from ranges
    let mut key = KeyBuffer::new();
    let mut first = true;
    
    for range in ranges_buf.iter() {
        if range.start >= range.end {
            // Empty field
            if !first {
                key.push(delimiter);
            }
        } else {
            if !first {
                key.push(delimiter);
            }
            key.extend_from_slice(&line[range.clone()]);
        }
        first = false;
    }
    JoinKey::Owned(key)
}

// Extracts values to append.
// We store them as a single byte string with delimiters, to avoid Vec<String> overhead.
fn extract_values(
    line: &[u8],
    delimiter: u8,
    plan: &crate::libs::select::SelectPlan,
    ranges_buf: &mut Vec<Range<usize>>,
) -> Vec<u8> {
    if let Err(idx) = plan.extract_ranges(line, delimiter, ranges_buf) {
        let n = if line.is_empty() {
            0
        } else {
            line.iter().filter(|&&b| b == delimiter).count() + 1
        };
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
    let delimiter_char = chars.next().unwrap_or('\t');
    let delimiter = delimiter_char as u8; // Assume single byte delimiter for now
    if chars.next().is_some() || delimiter_str.len() > 1 {
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

    // 1. Process Filter File
    let mut filter_reader = crate::libs::tsv::reader::TsvReader::new(
        crate::libs::io::reader(&filter_file)
    );

    let mut filter_header: Option<crate::libs::tsv::fields::Header> = None;
    if has_header {
        if let Some(header_bytes) = filter_reader.read_header()? {
            let header_line = String::from_utf8_lossy(&header_bytes);
            filter_header = Some(crate::libs::tsv::fields::Header::from_line(
                &header_line,
                delimiter_char,
            ));
        }
    }

    let (filter_key_whole_line, filter_key_indices) = parse_join_field_spec(
        key_fields_spec.clone(),
        filter_header.as_ref(),
        delimiter_char,
    );
    let append_indices =
        parse_append_field_spec(append_fields_spec, filter_header.as_ref(), delimiter_char);
    let append_count = append_indices.as_ref().map(|v| v.len()).unwrap_or(0);

    let filter_key_plan = filter_key_indices.as_ref().map(|idxs| crate::libs::select::SelectPlan::new(idxs));
    let append_plan = append_indices.as_ref().map(|idxs| crate::libs::select::SelectPlan::new(idxs));

    // Map: Key -> Appended Values (as bytes)
    let mut filter_map: HashMap<KeyBuffer, Vec<u8>, RandomState> = HashMap::with_hasher(RandomState::new());
    let mut ranges_buf: Vec<Range<usize>> = Vec::new(); // Reusable buffer for ranges

    filter_reader.for_each_record(|line| {
        if line.is_empty() {
            return Ok(());
        }
        let key = extract_key(line, delimiter, filter_key_whole_line, filter_key_plan.as_ref(), &mut ranges_buf);
        
        let values = if let Some(ref plan) = append_plan {
            extract_values(line, delimiter, plan, &mut ranges_buf)
        } else {
            Vec::new()
        };

        if let Some(existing) = filter_map.get_mut(key.as_ref()) {
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
            filter_map.insert(key.into_owned(), values);
        }
        Ok(())
    })?;

    // 2. Process Data Files
    let mut header_written = false;
    let prefix = args
        .get_one::<String>("prefix")
        .cloned()
        .unwrap_or_default();

    if !has_header && !prefix.is_empty() {
        arg_error("--prefix requires --header");
    }

    let mut data_key_whole_line = false;
    let mut data_key_plan: Option<crate::libs::select::SelectPlan> = None;
    let mut data_key_indices_len = 0; // For validation

    // Pre-calculate append header part
    let append_header_suffix = if let Some(idxs) = append_indices.as_ref() {
        if let Some(ref fh) = filter_header {
            let mut s = String::new();
            for idx in idxs {
                let pos = *idx - 1;
                if pos >= fh.fields.len() {
                    eprintln!(
                        "tva join: append index {} is out of range for filter header",
                        idx
                    );
                    std::process::exit(1);
                }
                s.push(delimiter_char);
                if prefix.is_empty() {
                    s.push_str(&fh.fields[pos]);
                } else {
                    s.push_str(&prefix);
                    s.push_str(&fh.fields[pos]);
                }
            }
            Some(s)
        } else {
            None
        }
    } else {
        None
    };

    let write_all_fill = write_all_value.as_ref().map(|fill| {
        let mut s = Vec::new();
        if append_count > 0 {
            for _ in 0..append_count {
                s.push(delimiter);
                s.extend_from_slice(fill.as_bytes());
            }
        }
        s
    });

    for input in crate::libs::io::raw_input_sources(&infiles) {
        // Use 128KB buffer for data reader
        let mut reader = crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 128 * 1024);
        let mut is_first_line = true;

        reader.for_each_record(|line| {
            if line.is_empty() {
                return Ok(());
            }

            if has_header && is_first_line {
                if !header_written {
                    let effective_data_spec =
                        data_fields_spec.clone().or_else(|| key_fields_spec.clone());
                    
                    let header_str = String::from_utf8_lossy(line);
                    let (whole_line, indices) = parse_join_field_spec(
                        effective_data_spec,
                        Some(&crate::libs::tsv::fields::Header::from_line(
                            &header_str, delimiter_char,
                        )),
                        delimiter_char,
                    );
                    
                    data_key_whole_line = whole_line;
                    if let Some(idxs) = indices {
                        data_key_indices_len = idxs.len();
                        data_key_plan = Some(crate::libs::select::SelectPlan::new(&idxs));
                    } else {
                        data_key_plan = None;
                    }

                    // Validate key lengths match
                    if !filter_key_whole_line && !data_key_whole_line {
                        let fk_len = filter_key_indices.as_ref().unwrap().len();
                        let dk_len = data_key_indices_len;
                        if fk_len != dk_len {
                            eprintln!(
                                "tva join: different number of key-fields and data-fields"
                            );
                            std::process::exit(1);
                        }
                    }

                    writer.write_all(line)?;
                    if let Some(ref suffix) = append_header_suffix {
                        writer.write_all(suffix.as_bytes())?;
                    }
                    writer.write_all(b"\n")?;
                    
                    if line_buffered {
                        writer.flush()?;
                    }
                    header_written = true;
                }
                is_first_line = false;
                return Ok(());
            }

            is_first_line = false;

            // Initialize plan for headerless files or subsequent files if not set
            if data_key_plan.is_none() && !data_key_whole_line {
                 let effective_data_spec =
                    data_fields_spec.clone().or_else(|| key_fields_spec.clone());
                let (whole_line, indices) =
                    parse_join_field_spec(effective_data_spec.clone(), None, delimiter_char);
                
                data_key_whole_line = whole_line;
                if let Some(idxs) = indices {
                    data_key_indices_len = idxs.len();
                    data_key_plan = Some(crate::libs::select::SelectPlan::new(&idxs));
                }

                if !filter_key_whole_line && !data_key_whole_line {
                    let fk_len = filter_key_indices.as_ref().unwrap().len();
                    let dk_len = data_key_indices_len;
                    if fk_len != dk_len {
                        eprintln!("tva join: different number of key-fields and data-fields");
                        std::process::exit(1);
                    }
                }
            }

            let key = extract_key(line, delimiter, data_key_whole_line, data_key_plan.as_ref(), &mut ranges_buf);
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
