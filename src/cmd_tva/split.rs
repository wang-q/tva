use clap::*;
use rapidhash::{rapidhash, RapidRng};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufWriter, Write};
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
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat first non-empty line as header and write it to every output file"),
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
}

struct SplitConfig<'a> {
    dir: &'a Path,
    prefix: &'a str,
    suffix: &'a str,
    digit_width: usize,
    append: bool,
    header_in_out: bool,
}

fn arg_error(msg: &str) -> ! {
    eprintln!("tva split: {}", msg);
    std::process::exit(1);
}

struct SplitOutput {
    writer: Box<dyn Write>,
    header_written: bool,
}

fn format_index(idx0: usize, digit_width: usize) -> String {
    let index = idx0 + 1;
    if digit_width == 0 {
        index.to_string()
    } else {
        format!("{:0width$}", index, width = digit_width)
    }
}

fn open_output_file(
    config: &SplitConfig,
    idx0: usize,
    header_line: Option<&str>,
) -> anyhow::Result<(Box<dyn Write>, bool)> {
    let index_str = format_index(idx0, config.digit_width);
    let filename = format!("{}{}{}", config.prefix, index_str, config.suffix);
    let path = config.dir.join(filename);
    let existed = path.exists();

    if existed && !config.append {
        return Err(anyhow::anyhow!(
            "tva split: output file already exists: {} (use --append/-a to append)",
            path.display()
        ));
    }

    let file: Box<dyn Write> = if config.append {
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
            if !(config.append && existed) {
                writer.write_all(header.as_bytes())?;
                writer.write_all(b"\n")?;
                header_written = true;
            } else {
                header_written = true;
            }
        }
    }

    Ok((writer, header_written))
}

fn get_or_create_output<'a>(
    outputs: &'a mut BTreeMap<usize, SplitOutput>,
    idx0: usize,
    config: &SplitConfig,
    header_line: Option<&'a str>,
) -> anyhow::Result<&'a mut SplitOutput> {
    if outputs.contains_key(&idx0) {
        return Ok(outputs.get_mut(&idx0).unwrap());
    }

    let (writer, header_written) = open_output_file(config, idx0, header_line)?;

    outputs.insert(
        idx0,
        SplitOutput {
            writer,
            header_written,
        },
    );

    Ok(outputs.get_mut(&idx0).unwrap())
}

fn parse_key_fields(spec: &str) -> Vec<usize> {
    crate::libs::tsv::fields::parse_numeric_field_list(spec)
        .unwrap_or_else(|e| arg_error(&e))
}

fn key_bucket(line: &str, indices: &[usize], num_files: usize) -> usize {
    let fields: Vec<&str> = line.split('\t').collect();
    let mut key_parts: Vec<&str> = Vec::new();
    for idx in indices {
        let pos = idx.saturating_sub(1);
        let value = fields.get(pos).copied().unwrap_or("");
        key_parts.push(value);
    }
    let key = key_parts.join("\t");
    let h = rapidhash(key.as_bytes());
    (h % (num_files as u64)) as usize
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let header_in_out = args.get_flag("header-in-out");
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
    let digit_width = args.get_one::<usize>("digit-width").cloned().unwrap_or(0);
    let append = args.get_flag("append");
    let static_seed = args.get_flag("static-seed");
    let seed_value = args.get_one::<u64>("seed-value").cloned().unwrap_or(0);

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
    };

    if lines_per_file > 0 {
        split_by_line_count(&infiles, &config, lines_per_file)?;
    } else {
        split_randomly(
            &infiles,
            &config,
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
    let mut header_line: Option<String> = None;
    let mut header_seen = false;

    let mut current_idx0: usize = 0;
    let mut current_writer: Option<Box<dyn Write>> = None;
    let mut current_lines: u64 = 0;

    for input in crate::libs::io::input_sources(infiles) {
        let reader = input.reader;
        let mut is_first_nonempty = true;

        for line in reader.lines().map_while(Result::ok) {
            let mut line = line;
            if let Some('\r') = line.chars().last() {
                line.pop();
            }

            if config.header_in_out
                && !header_seen
                && is_first_nonempty
                && !line.is_empty()
            {
                if header_line.is_none() {
                    header_line = Some(line.clone());
                }
                header_seen = true;
                is_first_nonempty = false;
                continue;
            }

            is_first_nonempty = false;

            if current_writer.is_none() || current_lines >= lines_per_file {
                let (writer, _) =
                    open_output_file(config, current_idx0, header_line.as_deref())?;
                current_writer = Some(writer);
                current_lines = 0;
                current_idx0 += 1;
            }

            if let Some(writer) = current_writer.as_mut() {
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
                current_lines += 1;
            }
        }
    }

    Ok(())
}

fn split_randomly(
    infiles: &[String],
    config: &SplitConfig,
    num_files: usize,
    key_indices: Option<&[usize]>,
    rng: &mut Option<RapidRng>,
) -> anyhow::Result<()> {
    if num_files == 0 {
        return Err(anyhow::anyhow!(
            "tva split: --num-files/-n must be greater than 0"
        ));
    }

    let mut header_line: Option<String> = None;
    let mut header_seen = false;

    let mut outputs: BTreeMap<usize, SplitOutput> = BTreeMap::new();

    for input in crate::libs::io::input_sources(infiles) {
        let reader = input.reader;
        let mut is_first_nonempty = true;

        for line in reader.lines().map_while(Result::ok) {
            let mut line = line;
            if let Some('\r') = line.chars().last() {
                line.pop();
            }

            if config.header_in_out
                && !header_seen
                && is_first_nonempty
                && !line.is_empty()
            {
                if header_line.is_none() {
                    header_line = Some(line.clone());
                }
                header_seen = true;
                is_first_nonempty = false;
                continue;
            }

            is_first_nonempty = false;

            let idx0 = if let Some(indices) = key_indices {
                key_bucket(&line, indices, num_files)
            } else {
                let r = rng
                    .as_mut()
                    .expect("random generator not initialized for split")
                    .next();
                (r as usize) % num_files
            };

            let output = get_or_create_output(
                &mut outputs,
                idx0,
                config,
                header_line.as_deref(),
            )?;

            if config.header_in_out && !output.header_written {
                if let Some(header) = header_line.as_deref() {
                    output.writer.write_all(header.as_bytes())?;
                    output.writer.write_all(b"\n")?;
                    output.header_written = true;
                }
            }

            output.writer.write_all(line.as_bytes())?;
            output.writer.write_all(b"\n")?;
        }
    }

    Ok(())
}
