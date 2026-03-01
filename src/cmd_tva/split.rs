use clap::*;
use indexmap::IndexMap;
use rapidhash::{rapidhash, RapidRng};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
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

struct SplitConfig<'a> {
    dir: &'a Path,
    prefix: &'a str,
    suffix: &'a str,
    digit_width: usize,
    append: bool,
    header_in_out: bool,
    header_in_only: bool,
    delimiter: u8,
}

fn arg_error(msg: &str) -> ! {
    eprintln!("tva split: {}", msg);
    std::process::exit(1);
}

struct SplitOutput {
    writer: Box<dyn Write>,
    #[allow(dead_code)]
    header_written: bool,
}

fn format_index(idx0: usize, digit_width: usize) -> String {
    let index = idx0; // 0-based indexing
    if digit_width == 0 {
        index.to_string()
    } else {
        format!("{:0width$}", index, width = digit_width)
    }
}

fn open_output_file(
    config: &SplitConfig,
    idx0: usize,
    header_line: Option<&[u8]>,
    force_append: bool,
) -> anyhow::Result<(Box<dyn Write>, bool)> {
    let index_str = format_index(idx0, config.digit_width);
    let filename = format!("{}{}{}", config.prefix, index_str, config.suffix);
    let path = config.dir.join(filename);
    let existed = path.exists();

    if existed && !force_append && !config.append {
        return Err(anyhow::anyhow!(
            "tva split: output file already exists: {} (use --append/-a to append)",
            path.display()
        ));
    }

    let is_append = config.append || force_append;
    let file: Box<dyn Write> = if is_append {
        Box::new(BufWriter::new(
            OpenOptions::new().create(true).append(true).open(&path)?,
        ))
    } else {
        Box::new(BufWriter::new(File::create(&path)?))
    };

    let mut writer = file;
    let mut header_written = false;

    if config.header_in_out {
        if let Some(header) = header_line {
            if !(is_append && existed) {
                writer.write_all(header)?;
                writer.write_all(b"\n")?;
                header_written = true;
            } else {
                header_written = true;
            }
        }
    }

    Ok((writer, header_written))
}

struct WriterManager<'a> {
    config: &'a SplitConfig<'a>,
    writers: IndexMap<usize, SplitOutput>,
    initialized_indices: HashSet<usize>,
    max_open: usize,
}

impl<'a> WriterManager<'a> {
    fn new(config: &'a SplitConfig<'a>, max_open: usize) -> Self {
        Self {
            config,
            writers: IndexMap::new(),
            initialized_indices: HashSet::new(),
            max_open,
        }
    }

    fn get_writer(
        &mut self,
        idx: usize,
        header_line: Option<&[u8]>,
    ) -> anyhow::Result<&mut SplitOutput> {
        if self.writers.contains_key(&idx) {
            // Move to end (MRU) if using LRU logic (max_open > 0)
            if self.max_open > 0 {
                if let Some((_, v)) = self.writers.shift_remove_entry(&idx) {
                    self.writers.insert(idx, v);
                }
            }
            return Ok(self.writers.get_mut(&idx).unwrap());
        }

        if self.max_open > 0 && self.writers.len() >= self.max_open {
            // Remove LRU (first item)
            self.writers.shift_remove_index(0);
        }

        let force_append = self.initialized_indices.contains(&idx);
        let (writer, header_written) =
            open_output_file(self.config, idx, header_line, force_append)?;

        self.initialized_indices.insert(idx);
        self.writers.insert(
            idx,
            SplitOutput {
                writer,
                header_written,
            },
        );

        Ok(self.writers.get_mut(&idx).unwrap())
    }
}

fn parse_key_fields(spec: &str) -> Vec<usize> {
    crate::libs::tsv::fields::parse_numeric_field_list(spec)
        .unwrap_or_else(|e| arg_error(&e))
}

fn key_bucket(
    record: &[u8],
    indices: &[usize],
    num_files: usize,
    delimiter: u8,
) -> usize {
    // Avoid allocating strings. Use zero-copy parsing.
    let mut key_buffer = Vec::new();
    let mut iter = memchr::memchr_iter(delimiter, record);
    let mut last_pos = 0;
    let mut field_idx = 1;
    let mut next_tab = iter.next();

    let mut first = true;

    for &target_idx in indices {
        // Advance to target_idx
        while field_idx < target_idx {
            if let Some(pos) = next_tab {
                last_pos = pos + 1;
                next_tab = iter.next();
                field_idx += 1;
            } else {
                // End of record
                break;
            }
        }

        if !first {
            key_buffer.push(b'\t'); // Internal key delimiter is always TAB? Or user delimiter?
                                    // tsv-uniq usually uses TAB as internal separator for multi-field keys.
                                    // Let's stick to TAB for consistency unless we want delimiter-agnostic key.
        }
        first = false;

        if field_idx == target_idx {
            let end = next_tab.unwrap_or(record.len());
            key_buffer.extend_from_slice(&record[last_pos..end]);
        }
        // If field_idx > target_idx (not found), push empty string (do nothing)
    }

    let h = rapidhash(&key_buffer);
    (h % (num_files as u64)) as usize
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
    let digit_width = if let Some(w) = args.get_one::<usize>("digit-width") {
        if *w == 0
            && args.value_source("digit-width")
                == Some(clap::parser::ValueSource::DefaultValue)
        {
            if lines_per_file > 0 {
                3
            } else if num_files > 0 {
                if num_files == 1 {
                    1
                } else {
                    let n = num_files as f64;
                    let w = n.log10().ceil() as usize;
                    if w == 0 {
                        1
                    } else {
                        w
                    }
                }
            } else {
                0
            }
        } else {
            *w
        }
    } else {
        0
    };
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

    let key_indices = if let Some(spec) = key_fields_spec {
        if spec.trim().is_empty() {
            None
        } else {
            Some(parse_key_fields(&spec))
        }
    } else {
        None
    };

    let config = SplitConfig {
        dir: &dir_path,
        prefix: &prefix,
        suffix: &suffix,
        digit_width,
        append,
        header_in_out,
        header_in_only,
        delimiter,
    };

    if lines_per_file > 0 {
        split_by_line_count(&infiles, &config, lines_per_file)?;
    } else {
        let mut manager = WriterManager::new(&config, max_open_files);
        split_randomly(
            &infiles,
            &mut manager,
            num_files,
            key_indices.as_deref(),
            &mut rng,
        )?;
    }

    Ok(())
}

fn split_by_line_count(
    infiles: &[String],
    config: &SplitConfig,
    lines_per_file: u64,
) -> anyhow::Result<()> {
    let mut header_line: Option<Vec<u8>> = None;
    let mut global_header_captured = false;

    let mut current_idx0: usize = 0;
    let mut current_writer: Option<Box<dyn Write>> = None;
    let mut current_lines: u64 = 0;

    for input in crate::libs::io::raw_input_sources(infiles) {
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

            if (config.header_in_out || config.header_in_only) && is_first_line_of_file {
                is_first_line_of_file = false;
                if !global_header_captured {
                    header_line = Some(record.to_vec());
                    global_header_captured = true;
                }
                return Ok(());
            }

            is_first_line_of_file = false;

            if current_writer.is_none() || current_lines >= lines_per_file {
                let (writer, _) = open_output_file(
                    config,
                    current_idx0,
                    header_line.as_deref(),
                    false,
                )
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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

    Ok(())
}

fn split_randomly(
    infiles: &[String],
    manager: &mut WriterManager,
    num_files: usize,
    key_indices: Option<&[usize]>,
    rng: &mut Option<RapidRng>,
) -> anyhow::Result<()> {
    if num_files == 0 {
        return Err(anyhow::anyhow!(
            "tva split: --num-files/-n must be greater than 0"
        ));
    }

    let mut header_line: Option<Vec<u8>> = None;
    let mut global_header_captured = false;

    for input in crate::libs::io::raw_input_sources(infiles) {
        let mut reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);
        let mut is_first_line_of_file = true;

        reader.for_each_record(|record| {
            if record.is_empty() {
                // Treat empty lines as data (will be assigned to a file)
                // But skip header check for them (as they are not header)
                // But we must NOT change is_first_line_of_file if we skip it?
                // No, empty lines are data.
                // But if the FIRST line is empty, it's not the header.
                // So we can proceed to assign it.
                // But `is_first_line_of_file` should remain true so we catch the REAL header later?
                // "Treat first NON-EMPTY line as header" implies we skip empty lines when looking for header.
                // So: if empty, write it, but don't toggle is_first_line_of_file?
                // But if we write it, it becomes part of the output.
                // If it's before the header, it's weird.
                // Let's assume standard behavior: process empty lines as data.
                // But if we haven't seen header yet, and we are looking for it...
                // If we output it, we might output data before header in the output file.
                // That's acceptable if the input has data before header.
            } else if (manager.config.header_in_out || manager.config.header_in_only)
                && is_first_line_of_file
            {
                is_first_line_of_file = false;
                if !global_header_captured {
                    header_line = Some(record.to_vec());
                    global_header_captured = true;
                }
                return Ok(());
            }

            // Only turn off flag if it was non-empty (handled in else if) OR if we decide empty lines count as "lines" that aren't header.
            // If we treat empty lines as data, they are not header.
            // If we encounter empty line at start, and we want header, we skip it?
            // "Treat first non-empty line as header".
            // So empty lines before header are... preserved? or skipped?
            // tsv-utils usually preserves everything.
            // If I have:
            // \n
            // Header
            // Data
            //
            // And I use --header.
            // Line 1: Empty. Not header. Process as data.
            // Line 2: Non-empty. Is header. Capture.
            // Line 3: Data.
            //
            // If I process Line 1 as data, I assign it to a file.
            // If that file is new, I write the header (which I haven't seen yet!).
            // So I write NULL header? Or no header?
            // `open_output_file` uses `header_line` (Option). If None, no header written.
            // So Line 1 goes to file X without header.
            // Line 2 is captured as header.
            // Line 3 goes to file Y. If Y is new, it gets Header.
            // This seems consistent with "streaming".

            // So `is_first_line_of_file` should ONLY be toggled if we found the header (non-empty).
            // BUT: if we process data, we are past the "start".
            // If we have data before header, that's a malformed TSV usually.
            // But let's stick to the logic: "first NON-EMPTY line".

            if !record.is_empty() {
                is_first_line_of_file = false;
            }

            // Wait, if I have 10 empty lines, then header.
            // 10 empty lines are processed. `is_first_line_of_file` remains true.
            // 11th line (Header) is processed. Matches `else if`. Captured. `is_first_line_of_file` = false.
            // 12th line (Data). Processed.

            // This looks correct.

            let idx0 = if let Some(indices) = key_indices {
                key_bucket(record, indices, num_files, manager.config.delimiter)
            } else {
                let r = rng
                    .as_mut()
                    .expect("random generator not initialized for split")
                    .next();
                (r as usize) % num_files
            };

            let output = manager
                .get_writer(idx0, header_line.as_deref())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            // If we are writing data, and we JUST found the header (e.g. data line came after header),
            // and the file was already open (from previous empty lines?),
            // we missed writing the header to that file!
            // `SplitOutput` has `header_written`.
            // If `header_written` is false, and we now have `header_line`, should we write it?
            // Only if we are at the top of the file?
            // But we already wrote empty lines to it.
            // Headers usually go at the top.
            // If we wrote empty lines, we can't insert header at top.
            // So: if data precedes header, the output will have data before header (or no header if file was opened before header known).
            // This is an edge case (malformed input). I won't over-engineer for it.

            // However, `SplitOutput` stores `header_written`.
            // If we open a file, we check `header_line`.

            output.writer.write_all(record)?;
            output.writer.write_all(b"\n")?;
            Ok(())
        })?;
    }

    Ok(())
}
