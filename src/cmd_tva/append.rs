use clap::*;
use std::io::Write;
use std::path::Path;

use crate::libs::tsv::reader::TsvReader;

pub fn make_subcommand() -> Command {
    Command::new("append")
        .about("Concatenates TSV files with optional header and source tracking")
        .after_help(include_str!("../../docs/help/append.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to append (default: stdin)"),
        )
        .arg(
            Arg::new("line-buffered")
                .long("line-buffered")
                .action(ArgAction::SetTrue)
                .help("Enable line-buffered output (flush after each line)"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line of each input as a header; only the first header is output"),
        )
        .arg(
            Arg::new("track-source")
                .long("track-source")
                .short('t')
                .action(ArgAction::SetTrue)
                .help("Add a source column indicating the originating file for each row"),
        )
        .arg(
            Arg::new("source-header")
                .long("source-header")
                .short('s')
                .num_args(1)
                .help("Header for the source column; implies --header and --track-source"),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Read FILE with explicit LABEL, as LABEL=FILE; implies --track-source"),
        )
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .short('d')
                .num_args(1)
                .default_value("\t")
                .help("Field delimiter to use when adding the source column (default: TAB)"),
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

struct InputSpec {
    path: String,
    label: Option<String>,
}

fn default_source_label(path: &str) -> String {
    if path == "stdin" || path == "-" {
        "stdin".to_string()
    } else {
        Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path)
            .to_string()
    }
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let line_buffered = args.get_flag("line-buffered");

    let mut has_header = args.get_flag("header");
    let mut track_source = args.get_flag("track-source");
    let source_header = args.get_one::<String>("source-header").cloned();

    // Check if --file was used (track_source implied)
    if args.contains_id("file") {
        track_source = true;
    }
    if source_header.is_some() {
        has_header = true;
        track_source = true;
    }

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .map(|s| s.as_str())
        .unwrap_or("\t");
    let delimiter_bytes = delimiter_str.as_bytes();
    if delimiter_bytes.len() != 1 {
        return Err(anyhow::anyhow!(
            "delimiter must be a single byte, got \"{}\"",
            delimiter_str
        ));
    }
    let delimiter = delimiter_bytes[0] as char;

    // Collect inputs respecting command line order
    let mut input_specs: Vec<(usize, InputSpec)> = Vec::new();

    // Positional args
    if let Some(indices) = args.indices_of("infiles") {
        if let Some(values) = args.get_many::<String>("infiles") {
            for (idx, val) in indices.zip(values) {
                input_specs.push((
                    idx,
                    InputSpec {
                        path: val.clone(),
                        label: None,
                    },
                ));
            }
        }
    }

    // Flag args (--file LABEL=FILE)
    if let Some(indices) = args.indices_of("file") {
        if let Some(values) = args.get_many::<String>("file") {
            for (idx, val) in indices.zip(values) {
                let raw = val.as_str();
                let mut parts = raw.splitn(2, '=');
                let label = parts.next().unwrap_or("");
                let file = parts.next().unwrap_or("");
                if label.is_empty() || file.is_empty() {
                    return Err(anyhow::anyhow!(
                        "invalid --file value `{}`; expected LABEL=FILE",
                        raw
                    ));
                }
                input_specs.push((
                    idx,
                    InputSpec {
                        path: file.to_string(),
                        label: Some(label.to_string()),
                    },
                ));
            }
        }
    }

    // Sort by index to preserve command line order
    input_specs.sort_by_key(|k| k.0);

    let mut specs: Vec<InputSpec> =
        input_specs.into_iter().map(|(_, spec)| spec).collect();

    // Default to stdin if no inputs
    if specs.is_empty() {
        specs.push(InputSpec {
            path: "stdin".to_string(),
            label: None,
        });
    }

    let source_header_name = source_header.unwrap_or_else(|| "file".to_string());
    let mut header_written = false;

    for spec in specs {
        let input_reader = crate::libs::io::raw_reader(&spec.path);
        let mut reader = TsvReader::new(input_reader);

        let label = match spec.label {
            Some(ref s) => s.clone(),
            None => default_source_label(&spec.path),
        };
        let label_bytes = label.as_bytes();

        if has_header {
            if let Some(header) = reader.read_header()? {
                if !header_written {
                    header_written = true;
                    if track_source {
                        writer.write_all(source_header_name.as_bytes())?;
                        writer.write_all(&[delimiter as u8])?;
                        writer.write_all(&header)?;
                        writer.write_all(b"\n")?;
                    } else {
                        writer.write_all(&header)?;
                        writer.write_all(b"\n")?;
                    }
                    if line_buffered {
                        writer.flush()?;
                    }
                }
            }
        }

        reader.for_each_record(|record| {
            if track_source {
                writer.write_all(label_bytes)?;
                writer.write_all(&[delimiter as u8])?;
            }
            writer.write_all(record)?;
            writer.write_all(b"\n")?;

            if line_buffered {
                writer.flush()?;
            }

            Ok(())
        })?;
    }

    Ok(())
}
