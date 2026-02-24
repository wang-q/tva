use clap::*;
use rapidhash::rapidhash;
use std::collections::HashMap;
use std::io::{BufRead, Write};

pub fn make_subcommand() -> Command {
    Command::new("uniq")
        .about("Deduplicates TSV rows from one or more files")
        .after_help(
            r###"
Deduplicates rows of one or more tab-separated values (TSV) files without sorting.

Notes:
* Supports plain text and gzipped (.gz) TSV files
* Reads from stdin if no input file is given or if input file is 'stdin'
* Keeps a 64-bit hash for each unique key; ~8 bytes of memory per unique row
* Only the first occurrence of each key is kept; occurrences are not counted

Field syntax:
- When --header is given, --fields/-f accepts 1-based indices, ranges
  (1-3,5-7), header names, name ranges (run-user_time), and wildcards (*_time).
- Run `tva --help-fields` for a full description shared across tva commands.

Examples:
1. Deduplicate whole rows
   tva uniq tests/genome/ctg.tsv

2. Deduplicate by column 2
   tva uniq tests/genome/ctg.tsv -f 2
"###,
        )
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
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line of each input as a header; only the first header is output"),
        )
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

fn arg_error(msg: &str) -> ! {
    eprintln!("tva uniq: {}", msg);
    std::process::exit(1);
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let fields_spec: Option<String> = args.get_one::<String>("fields").cloned();

    let has_header = args.get_flag("header");
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

    if repeated && at_least <= 1 {
        at_least = 2;
    }
    if at_least >= 2 && max < at_least {
        if max != 0 || (!equiv_mode && !number_mode) {
            max = at_least;
        }
    }

    if !equiv_mode {
        if args.get_one::<String>("equiv-header").is_some() {
            arg_error("--equiv-header requires --equiv");
        }
        if args.get_one::<String>("equiv-start").is_some() {
            arg_error("--equiv-start requires --equiv");
        }
    }

    if !number_mode {
        if args.get_one::<String>("number-header").is_some() {
            arg_error("--number-header requires --number");
        }
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
        1
    } else {
        equiv_start as u64
    };

    let mut header_written = false;
    let mut header: Option<crate::libs::fields::Header> = None;
    let mut key_fields: Option<Vec<usize>> = None;

    struct EquivEntry {
        equiv_id: u64,
        count: u64,
    }

    let mut equiv_map: HashMap<u64, EquivEntry> = HashMap::new();

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;
        let mut is_first_line = true;

        for line in reader.lines().map_while(Result::ok) {
            if has_header && is_first_line {
                if header.is_none() {
                    header = Some(crate::libs::fields::Header::from_line(&line, delimiter));
                    if let Some(ref spec) = fields_spec {
                        if spec.trim() == "0" {
                            key_fields = Some(Vec::new());
                        } else {
                            let parsed =
                                crate::libs::fields::parse_field_list_with_header(
                                    spec,
                                    header.as_ref(),
                                    delimiter,
                                );
                            match parsed {
                                Ok(v) => key_fields = Some(v),
                                Err(e) => {
                                arg_error(&e);
                                }
                            }
                        }
                    }
                }

                if !header_written {
                    let mut header_line = line.clone();
                    if equiv_mode {
                        header_line.push(delimiter);
                        header_line.push_str(&equiv_header);
                    }
                    if number_mode {
                        header_line.push(delimiter);
                        header_line.push_str(&number_header);
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

            if key_fields.is_none() {
                if let Some(ref spec) = fields_spec {
                    if spec.trim() == "0" {
                        key_fields = Some(Vec::new());
                    } else {
                        let parsed =
                            crate::libs::fields::parse_field_list_with_header(
                                spec,
                                None,
                                delimiter,
                            );
                        match parsed {
                            Ok(v) => key_fields = Some(v),
                            Err(e) => {
                                arg_error(&e);
                            }
                        }
                    }
                }
            }

            let subject = if key_fields.as_ref().map_or(true, |v| v.is_empty()) {
                if ignore_case {
                    let lower = line.to_lowercase();
                    rapidhash(lower.as_bytes())
                } else {
                    rapidhash(line.as_bytes())
                }
            } else {
                let fields: Vec<&str> = line.split(delimiter).collect();
                let subset: Vec<&str> = key_fields
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter_map(|&i| fields.get(i - 1))
                    .copied()
                    .collect();
                let concat = subset.join(&delimiter.to_string());
                if ignore_case {
                    let lower = concat.to_lowercase();
                    rapidhash(lower.as_bytes())
                } else {
                    rapidhash(concat.as_bytes())
                }
            };

            let entry = equiv_map.entry(subject).or_insert_with(|| {
                let id = next_equiv_id;
                next_equiv_id += 1;
                EquivEntry { equiv_id: id, count: 0 }
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
                if !equiv_mode && !number_mode {
                    writer.write_fmt(format_args!("{}\n", line))?;
                } else {
                    let mut out_line = line.clone();
                    if equiv_mode {
                        out_line.push(delimiter);
                        out_line.push_str(&entry.equiv_id.to_string());
                    }
                    if number_mode {
                        out_line.push(delimiter);
                        out_line.push_str(&entry.count.to_string());
                    }
                    writer.write_fmt(format_args!("{}\n", out_line))?;
                }
                if line_buffered {
                    writer.flush()?;
                }
            }
        }
    }

    Ok(())
}
