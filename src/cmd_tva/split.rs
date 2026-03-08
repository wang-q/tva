use clap::*;
use rapidhash::{rapidhash, RapidRng};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn make_subcommand() -> Command {
    Command::new("split")
        .about("Splits TSV rows into multiple files")
        .after_help(include_str!("../../docs/help/split.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to split (default: stdin)"),
        )
        .arg(
            Arg::new("header-in-out")
                .long("header-in-out")
                .visible_alias("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat first non-empty line as header and write it to every output file"),
        )
        .arg(
            Arg::new("header-in-only")
                .long("header-in-only")
                .short('I')
                .action(ArgAction::SetTrue)
                .conflicts_with("header-in-out")
                .help("Treat first non-empty line as header and do NOT write it to output files"),
        )
        .arg(
            Arg::new("lines-per-file")
                .long("lines-per-file")
                .short('l')
                .num_args(1)
                .value_parser(value_parser!(u64))
                .help("Number of data rows per output file (0 means disabled)"),
        )
        .arg(
            Arg::new("num-files")
                .long("num-files")
                .short('n')
                .num_args(1)
                .value_parser(value_parser!(usize))
                .help("Number of output files for random assignment"),
        )
        .arg(
            Arg::new("key-fields")
                .long("key-fields")
                .short('k')
                .num_args(1)
                .help("Numeric field list (1-based, ranges allowed) used as key for random assignment by key"),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .short('d')
                .num_args(1)
                .default_value(".")
                .help("Output directory for split files"),
        )
        .arg(
            Arg::new("prefix")
                .long("prefix")
                .num_args(1)
                .default_value("split-")
                .help("Prefix for output file names"),
        )
        .arg(
            Arg::new("suffix")
                .long("suffix")
                .num_args(1)
                .default_value(".tsv")
                .help("Suffix for output file names"),
        )
        .arg(
            Arg::new("digit-width")
                .long("digit-width")
                .short('w')
                .num_args(1)
                .value_parser(value_parser!(usize))
                .default_value("0")
                .help("Zero-padding width for numeric file index (0 disables padding)"),
        )
        .arg(
            Arg::new("append")
                .long("append")
                .short('a')
                .action(ArgAction::SetTrue)
                .help("Append to existing output files instead of overwriting"),
        )
        .arg(
            Arg::new("static-seed")
                .long("static-seed")
                .short('s')
                .action(ArgAction::SetTrue)
                .help("Use a fixed random seed so repeated runs produce the same result"),
        )
        .arg(
            Arg::new("seed-value")
                .long("seed-value")
                .short('v')
                .num_args(1)
                .value_parser(value_parser!(u64))
                .help("Explicit 64-bit seed value (non-zero) for the random generator"),
        )
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .visible_alias("delim")
                .num_args(1)
                .default_value("\t")
                .help("Field delimiter"),
        )
        .arg(
            Arg::new("max-open-files")
                .long("max-open-files")
                .num_args(1)
                .value_parser(value_parser!(usize))
                .help("Maximum number of open file handles"),
        )
}

fn arg_error(msg: &str) -> ! {
    eprintln!("tva split: {}", msg);
    std::process::exit(1);
}

fn format_index(idx0: usize, digit_width: usize) -> String {
    if digit_width == 0 {
        idx0.to_string()
    } else {
        format!("{:0width$}", idx0, width = digit_width)
    }
}



pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let header_in_out = args.get_flag("header-in-out");
    let header_in_only = args.get_flag("header-in-only");
    let lines_per_file = args.get_one::<u64>("lines-per-file").cloned().unwrap_or(0);
    let num_files = args.get_one::<usize>("num-files").cloned().unwrap_or(0);
    let key_fields_spec = args.get_one::<String>("key-fields").map(|s| s.to_string());
    let dir_str = args
        .get_one::<String>("dir")
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    let prefix = args
        .get_one::<String>("prefix")
        .cloned()
        .unwrap_or_else(|| "split-".to_string());
    let suffix = args
        .get_one::<String>("suffix")
        .cloned()
        .unwrap_or_else(|| ".tsv".to_string());
    // Calculate default digit width based on mode when user doesn't specify
    let digit_width = args
        .get_one::<usize>("digit-width")
        .copied()
        .map(|w| {
            if w != 0 {
                // User explicitly specified a non-zero value
                w
            } else if args.value_source("digit-width")
                != Some(clap::parser::ValueSource::DefaultValue)
            {
                // User explicitly specified 0 (disable padding)
                0
            } else {
                // Default value: auto-calculate based on mode
                match (lines_per_file, num_files) {
                    (_, 1) => 1,
                    (0, n) if n > 1 => ((n as f64).log10().ceil() as usize).max(1),
                    (l, 0) if l > 0 => 3, // lines-per-file mode: default 3 digits
                    _ => 0,
                }
            }
        })
        .unwrap_or(0);
    let append = args.get_flag("append");
    let static_seed = args.get_flag("static-seed");
    let seed_value = args.get_one::<u64>("seed-value").cloned().unwrap_or(0);
    let delimiter_str = args
        .get_one::<String>("delimiter")
        .map(|s| s.as_str())
        .unwrap_or("\t");
    let delimiter = if delimiter_str == "\\t" {
        b'\t'
    } else {
        delimiter_str.as_bytes()[0]
    };
    let max_open_files = args
        .get_one::<usize>("max-open-files")
        .cloned()
        .unwrap_or(0);

    if lines_per_file == 0 && num_files == 0 {
        arg_error("either --lines-per-file/-l or --num-files/-n must be specified");
    }

    if lines_per_file > 0 && num_files > 0 {
        arg_error("--lines-per-file/-l cannot be used with --num-files/-n");
    }

    if lines_per_file > 0 && key_fields_spec.is_some() {
        arg_error("--key-fields/-k is only supported with --num-files/-n");
    }

    if num_files == 0 && key_fields_spec.is_some() {
        arg_error("--key-fields/-k requires --num-files/-n");
    }

    let dir_path = PathBuf::from(&dir_str);
    if dir_path.exists() {
        let meta = fs::metadata(&dir_path)?;
        if !meta.is_dir() {
            return Err(anyhow::anyhow!(
                "tva split: output path is not a directory: {}",
                dir_path.display()
            ));
        }
    } else {
        fs::create_dir_all(&dir_path)?;
    }

    let mut rng = if num_files > 0 && key_fields_spec.is_none() {
        if !static_seed && seed_value == 0 {
            Some(RapidRng::default())
        } else if seed_value != 0 {
            Some(RapidRng::new(seed_value))
        } else {
            Some(RapidRng::new(2438424139))
        }
    } else {
        None
    };

    let key_indices = key_fields_spec.and_then(|spec| {
        if spec.trim().is_empty() {
            None
        } else {
            Some(
                crate::libs::tsv::fields::parse_numeric_field_list(&spec)
                    .unwrap_or_else(|e| arg_error(&e)),
            )
        }
    });

    // Create KeyExtractor once for key-based splitting
    let mut key_extractor = key_indices.as_ref().map(|indices| {
        crate::libs::tsv::key::KeyExtractor::new(Some(indices.clone()), false, false)
    });

    if lines_per_file > 0 {
        split_by_line_count(
            &infiles,
            &dir_path,
            &prefix,
            &suffix,
            digit_width,
            header_in_out,
            header_in_only,
            lines_per_file,
            append,
        )?;
    } else {
        let mut manager =
            crate::libs::io::FileWriterManager::new(&dir_path, max_open_files);

        // In append mode, mark existing files as initialized
        if append {
            let existing_indices: Vec<usize> = (0..num_files)
                .filter(|&i| {
                    let filename =
                        format!("{}{}{}", prefix, format_index(i, digit_width), suffix);
                    dir_path.join(&filename).exists()
                })
                .collect();
            manager.mark_initialized(&existing_indices);
        }

        split_randomly(
            &infiles,
            &mut manager,
            &prefix,
            &suffix,
            digit_width,
            header_in_out,
            header_in_only,
            num_files,
            key_extractor.as_mut(),
            &mut rng,
            delimiter,
            append,
        )?;
    }

    Ok(())
}

fn split_by_line_count(
    infiles: &[String],
    dir: &Path,
    prefix: &str,
    suffix: &str,
    digit_width: usize,
    header_in_out: bool,
    header_in_only: bool,
    lines_per_file: u64,
    append: bool,
) -> anyhow::Result<()> {
    let mut header_line: Option<Vec<u8>> = None;
    let mut global_header_captured = false;

    let mut current_idx0: usize = 0;
    let mut current_writer: Option<std::io::BufWriter<std::fs::File>> = None;
    let mut current_lines: u64 = 0;

    for input in crate::libs::io::raw_input_sources(infiles)? {
        let mut reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);
        let mut is_first_line_of_file = true;

        reader.for_each_record(|record| {
            if record.is_empty() {
                if let Some(writer) = current_writer.as_mut() {
                    writer.write_all(b"\n")?;
                    current_lines += 1;
                }
                return Ok(());
            }

            if (header_in_out || header_in_only) && is_first_line_of_file {
                is_first_line_of_file = false;
                if !global_header_captured {
                    header_line = Some(record.to_vec());
                    global_header_captured = true;
                }
                return Ok(());
            }

            is_first_line_of_file = false;

            if current_writer.is_none() || current_lines >= lines_per_file {
                // Close previous writer
                if let Some(mut w) = current_writer.take() {
                    w.flush()?;
                }

                // Open new file
                let filename = format!("{}{}{}", prefix, format_index(current_idx0, digit_width), suffix);
                let path = dir.join(&filename);
                let existed = path.exists();

                // Check for existing file (only for first file when not appending)
                if existed && !append && current_idx0 == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        format!(
                            "tva split: output file already exists: {} (use --append/-a to append)",
                            path.display()
                        ),
                    ));
                }

                let file = if append && existed {
                    std::fs::OpenOptions::new().create(true).append(true).open(&path)?
                } else {
                    std::fs::File::create(&path)?
                };
                let mut writer = std::io::BufWriter::new(file);

                // Write header if needed (only for new files, not when appending)
                if header_in_out && !(append && existed) {
                    if let Some(h) = header_line.as_ref() {
                        writer.write_all(h)?;
                        writer.write_all(b"\n")?;
                    }
                }

                current_writer = Some(writer);
                current_lines = 0;
                current_idx0 += 1;
            }

            if let Some(writer) = current_writer.as_mut() {
                writer.write_all(record)?;
                writer.write_all(b"\n")?;
                current_lines += 1;
            }
            Ok(())
        })?;
    }

    // Flush final writer
    if let Some(mut w) = current_writer {
        w.flush()?;
    }

    Ok(())
}

fn split_randomly(
    infiles: &[String],
    manager: &mut crate::libs::io::FileWriterManager,
    prefix: &str,
    suffix: &str,
    digit_width: usize,
    header_in_out: bool,
    header_in_only: bool,
    num_files: usize,
    mut key_extractor: Option<&mut crate::libs::tsv::key::KeyExtractor>,
    rng: &mut Option<RapidRng>,
    delimiter: u8,
    append: bool,
) -> anyhow::Result<()> {
    if num_files == 0 {
        return Err(anyhow::anyhow!(
            "tva split: --num-files/-n must be greater than 0"
        ));
    }

    let mut header_line: Option<Vec<u8>> = None;
    let mut global_header_captured = false;
    let mut checked_existing = false;

    for input in crate::libs::io::raw_input_sources(infiles)? {
        let mut reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);
        let mut is_first_line_of_file = true;

        reader.for_each_record(|record| {
            // Treat empty lines as data (will be assigned to a file).
            if !record.is_empty()
                && (header_in_out || header_in_only)
                && is_first_line_of_file
            {
                is_first_line_of_file = false;
                if !global_header_captured {
                    header_line = Some(record.to_vec());
                    global_header_captured = true;
                }
                return Ok(());
            }

            if !record.is_empty() {
                is_first_line_of_file = false;
            }

            let idx0 = if let Some(extractor) = key_extractor.as_mut() {
                let key = extractor.extract(record, delimiter).unwrap_or_else(|_| {
                    crate::libs::tsv::key::ParsedKey::Ref(b"")
                });
                (rapidhash(key.as_ref()) % (num_files as u64)) as usize
            } else {
                let r = rng
                    .as_mut()
                    .expect("random generator not initialized for split")
                    .next();
                (r as usize) % num_files
            };

            // Check for existing files on first write (lazy check, only if not appending)
            if !checked_existing && !append {
                checked_existing = true;
                for i in 0..num_files {
                    let filename = format!("{}{}{}", prefix, format_index(i, digit_width), suffix);
                    let path = manager.dir().join(&filename);
                    if path.exists() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::AlreadyExists,
                            format!(
                                "tva split: output file already exists: {} (use --append/-a to append)",
                                path.display()
                            ),
                        ));
                    }
                }
            }

            let writer = manager
                .get_writer_with_header(
                    idx0,
                    prefix,
                    suffix,
                    if header_in_out { header_line.as_deref() } else { None },
                )
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            writer.write_all(record)?;
            writer.write_all(b"\n")?;
            Ok(())
        })?;
    }

    manager.flush_all()?;
    Ok(())
}
