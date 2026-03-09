use crate::libs::cli::{build_header_config, header_args_with_columns};
use clap::*;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("nl")
        .about("Adds line numbers to TSV rows")
        .after_help(include_str!("../../docs/help/nl.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .args(header_args_with_columns())
        .arg(
            Arg::new("header-string")
                .long("header-string")
                .short('s')
                .num_args(1)
                .help("Header for the line number column; implies --header"),
        )
        .arg(
            Arg::new("start-number")
                .long("start-number")
                .short('n')
                .num_args(1)
                .default_value("1")
                .value_parser(value_parser!(i64))
                .allow_hyphen_values(true)
                .help("Number to use for the first line (can be negative)"),
        )
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .short('d')
                .num_args(1)
                .default_value("\t")
                .help("Delimiter between the line number and the line content"),
        )
        .arg(
            Arg::new("line-buffered")
                .long("line-buffered")
                .action(ArgAction::SetTrue)
                .help("Force line-buffered output mode. Useful for real-time viewing with `tail -f`"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let line_buffered = args.get_flag("line-buffered");

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    // Build HeaderConfig from arguments
    let mut header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    let header_string = if let Some(s) = args.get_one::<String>("header-string") {
        // header-string implies --header
        header_config.enabled = true;
        s.clone()
    } else {
        "line".to_string()
    };

    let mut line_num: i64 = *args.get_one::<i64>("start-number").unwrap();
    let mut header_written = false;
    let delimiter = args
        .get_one::<String>("delimiter")
        .map(|s| s.as_str())
        .unwrap_or("\t");
    let delimiter_bytes = delimiter.as_bytes();
    let header_bytes = header_string.as_bytes();

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);

        // If header is enabled, read header according to the configured mode
        if header_config.enabled {
            let header_result = reader
                .read_header_mode(header_config.mode)
                .map_err(|e| anyhow::anyhow!(e))?;

            if let Some(header_info) = header_result {
                // Write header only for the first file
                if !header_written {
                    // Write all header lines (hash lines, or LinesN lines)
                    for line in &header_info.lines {
                        writer.write_all(line)?;
                        writer.write_all(b"\n")?;
                    }
                    // For modes that provide column names, write the modified header line
                    if let Some(column_names) = header_info.column_names_line {
                        writer.write_all(header_bytes)?;
                        writer.write_all(delimiter_bytes)?;
                        writer.write_all(&column_names)?;
                        writer.write_all(b"\n")?;
                    }
                    writer.flush()?;
                    header_written = true;
                }
            }
        }

        // Process remaining data rows
        // Empty files are naturally skipped as for_each_record finds no records
        reader.for_each_record(|line| {
            writer.write_all(line_num.to_string().as_bytes())?;
            writer.write_all(delimiter_bytes)?;
            writer.write_all(line)?;
            writer.write_all(b"\n")?;
            if line_buffered {
                writer.flush()?;
            }
            line_num += 1;
            Ok(())
        })?;
    }

    Ok(())
}
