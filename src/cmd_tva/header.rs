use clap::*;
use std::collections::BTreeMap;
use std::io::Write;

use crate::libs::cli::get_delimiter;
use crate::libs::io::map_io_err;
use crate::libs::tsv::reader::TsvReader;

pub fn make_subcommand() -> Command {
    Command::new("header")
        .about("Prints the headers of TSV files")
        .after_help(include_str!("../../docs/help/header.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("names-only")
                .long("names-only")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Only show header names (hide column index)"),
        )
        .arg(
            Arg::new("start")
                .long("start")
                .short('s')
                .num_args(1)
                .default_value("1")
                .value_parser(value_parser!(usize))
                .help("Column indices will start from given number"),
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

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let names_only = args.get_flag("names-only");
    let start_idx = *args.get_one::<usize>("start").unwrap();
    let opt_delimiter = get_delimiter(args, "delimiter")?;

    let mut headers_per_input: Vec<(String, Vec<String>)> =
        Vec::with_capacity(infiles.len());

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut reader = TsvReader::new(input.reader);

        let first_line = reader
            .read_header()
            .map_err(map_io_err)?
            .ok_or_else(|| anyhow::anyhow!("empty file: {}", input.name))?;

        let first_line_str = String::from_utf8_lossy(&first_line);
        let headers: Vec<String> =
            first_line_str.split('\t').map(|s| s.to_string()).collect();

        let display_path = if input.is_stdin {
            "<stdin>".to_string()
        } else {
            input.name
        };

        headers_per_input.push((display_path, headers));
    }

    output_headers(
        &mut writer,
        &headers_per_input,
        names_only,
        start_idx,
        opt_delimiter,
    )?;

    Ok(())
}

fn output_headers(
    writer: &mut dyn Write,
    headers_per_input: &[(String, Vec<String>)],
    names_only: bool,
    start_idx: usize,
    delimiter: u8,
) -> anyhow::Result<()> {
    let max_len = headers_per_input
        .iter()
        .map(|(_, h)| h.len())
        .max()
        .unwrap_or(0);

    // Write header row
    if !names_only {
        writer.write_all(b"file")?;
        for (path, _) in headers_per_input {
            writer.write_all(&[delimiter])?;
            writer.write_all(path.as_bytes())?;
        }
        writer.write_all(b"\n")?;
    } else {
        // names_only: just write file paths as headers
        for (i, (path, _)) in headers_per_input.iter().enumerate() {
            if i > 0 {
                writer.write_all(&[delimiter])?;
            }
            writer.write_all(path.as_bytes())?;
        }
        writer.write_all(b"\n")?;
    }

    // Build name counts for diverging detection
    let mut name_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, headers) in headers_per_input {
        for name in headers {
            *name_counts.entry(name.clone()).or_insert(0) += 1;
        }
    }

    // Find duplicates for each file
    let duplicates_per_file: Vec<Vec<String>> = headers_per_input
        .iter()
        .map(|(_, headers)| find_duplicates(headers))
        .collect();

    // Write each row
    for row_idx in 0..max_len {
        if !names_only {
            let idx_str = (start_idx + row_idx).to_string();
            writer.write_all(idx_str.as_bytes())?;
        }

        for (file_idx, (_, headers)) in headers_per_input.iter().enumerate() {
            if !names_only || file_idx > 0 {
                writer.write_all(&[delimiter])?;
            }

            if let Some(h) = headers.get(row_idx) {
                let is_duplicate = duplicates_per_file[file_idx].contains(h);
                let is_diverging =
                    name_counts.get(h).copied().unwrap_or(0) < headers_per_input.len();

                let mut display = h.clone();
                if is_duplicate {
                    display.push_str(" [duplicate]");
                } else if headers_per_input.len() > 1 && is_diverging {
                    display.push_str(" [diverging]");
                }

                writer.write_all(display.as_bytes())?;
            }
        }
        writer.write_all(b"\n")?;
    }

    writer.flush()?;
    Ok(())
}

fn find_duplicates(headers: &[String]) -> Vec<String> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();

    for h in headers.iter() {
        *counts.entry(h).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .filter(|(_, c)| *c > 1)
        .map(|(h, _)| h.to_string())
        .collect()
}
