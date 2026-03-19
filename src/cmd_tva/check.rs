use clap::*;

use crate::libs::cli::{build_header_config, header_args};
use crate::libs::io::map_io_err;

pub fn make_subcommand() -> Command {
    Command::new("check")
        .about("Checks TSV table structure for consistent field counts")
        .after_help(include_str!("../../docs/help/check.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to check (default: stdin)"),
        )
        .args(header_args())
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;
    let has_header = header_config.enabled;

    let mut total_lines: u64 = 0;
    let mut total_data_lines: u64 = 0;
    let mut expected_fields: Option<usize> = None;

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);

        if has_header {
            let header_result = reader
                .read_header_mode(header_config.mode)
                .map_err(map_io_err)?;

            if let Some(header_info) = header_result {
                // Count header lines (hash lines or LinesN lines)
                total_lines += header_info.lines.len() as u64;

                // For modes that provide column names, also count the column names line
                if let Some(ref column_names_line) = header_info.column_names_line {
                    total_lines += 1;

                    // Count fields in column names line
                    let header_fields = if column_names_line.is_empty() {
                        0
                    } else {
                        memchr::memchr_iter(b'\t', column_names_line).count() + 1
                    };

                    if expected_fields.is_none() {
                        expected_fields = Some(header_fields);
                    }
                }
            }
        }

        reader.for_each_line(|record| {
            total_lines += 1;
            total_data_lines += 1;

            let fields = if record.is_empty() {
                0
            } else {
                memchr::memchr_iter(b'\t', record).count() + 1
            };

            if let Some(exp) = expected_fields {
                if fields != exp {
                    eprintln!("line {} ({} fields):", total_lines, fields);
                    // Lossy conversion for error display is fine
                    eprintln!("  {}", String::from_utf8_lossy(record));
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "tva check: structure check failed: line {} has {} fields (expected {})",
                            total_lines, fields, exp
                        ),
                    ));
                }
            } else {
                expected_fields = Some(fields);
            }
            Ok(())
        })?;
    }

    match expected_fields {
        Some(fields) => {
            if has_header {
                println!(
                    "{} lines total, {} data lines, {} fields",
                    total_lines, total_data_lines, fields
                );
            } else {
                println!("{} lines, {} fields", total_lines, fields);
            }
        }
        None => {
            println!("0 lines, 0 fields");
        }
    }

    Ok(())
}
