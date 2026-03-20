use crate::libs::cli::{build_header_config, header_args};
use crate::libs::tsv::header::{write_header, Header};
use clap::*;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("slice")
        .about("Slice rows by index (keep or drop)")
        .after_help(include_str!("../../docs/help/slice.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("rows")
                .long("rows")
                .short('r')
                .action(ArgAction::Append)
                .allow_hyphen_values(true) // Allow negative values (e.g., "-5")
                .help("Row range(s) to select (e.g. 5, 1-5, 10-). 1-based."),
        )
        .arg(
            Arg::new("invert")
                .long("invert")
                .short('v')
                .action(ArgAction::SetTrue)
                .help("Invert selection: drop selected rows instead of keeping them"),
        )
        .args(header_args())
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
}

#[derive(Debug, Clone)]
struct Range {
    start: usize, // 1-based
    end: usize,   // 1-based, usize::MAX means end of file
}

fn parse_range(s: &str) -> anyhow::Result<Range> {
    // Check for negative number (e.g., "-2") which might be interpreted as flag by clap if not careful,
    // but here we get it as string.
    // However, `split_once('-')` on "-2" gives:
    // start_str = "", end_str = "2"
    // My logic: start = 1, end = 2. This is correct for "-2" meaning "1-2".

    if let Some((start_str, end_str)) = s.split_once('-') {
        // Range: N-M, N-, -M
        let start = if start_str.is_empty() {
            1
        } else {
            start_str.parse::<usize>()?
        };

        let end = if end_str.is_empty() {
            usize::MAX
        } else {
            end_str.parse::<usize>()?
        };

        if start == 0 {
            return Err(anyhow::anyhow!("Row index must be >= 1"));
        }
        if end < start {
            return Err(anyhow::anyhow!("Invalid range: end < start"));
        }

        Ok(Range { start, end })
    } else {
        // Single row: N
        let n = s.parse::<usize>()?;
        if n == 0 {
            return Err(anyhow::anyhow!("Row index must be >= 1"));
        }
        Ok(Range { start: n, end: n })
    }
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let invert = args.get_flag("invert");

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    let mut ranges = Vec::new();
    if let Some(values) = args.get_many::<String>("rows") {
        for v in values {
            ranges.push(parse_range(v)?);
        }
    }

    // Optimization: Merge overlapping ranges if necessary?
    // For now, simple check is enough.
    // Actually, sorting ranges by start might help performance if we have many ranges.
    ranges.sort_by_key(|r| r.start);

    let mut header_written = false;

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);
        let mut line_num;
        let mut header_lines_count = 0;

        // If header is enabled, read header according to the configured mode
        if header_config.enabled {
            let header_result = reader
                .read_header_mode(header_config.mode)
                .map_err(|e| anyhow::anyhow!(e))?;

            if let Some(header_info) = header_result {
                // Count header lines to adjust line numbers
                // This ensures row indices in ranges refer to absolute line numbers
                header_lines_count = header_info.lines.len();
                if header_info.column_names_line.is_some() {
                    header_lines_count += 1;
                }

                // Write header only for the first file
                if !header_written {
                    let header = Header::from_info(header_info, '\t');
                    write_header(&mut writer, &header, None)?;
                    header_written = true;
                }
            }
        }

        // Initialize line_num to header_lines_count so that data rows
        // have correct absolute line numbers
        line_num = header_lines_count;

        reader.for_each_line(|record| {
            line_num += 1;

            // Check if current line is in any range
            let mut in_range = false;

            // If ranges list is empty:
            // - If invert is false (Keep mode): keep nothing (except header if -H) -> standard behavior
            // - If invert is true (Drop mode): drop nothing -> keep all
            if ranges.is_empty() {
                if invert {
                    // Drop nothing = Keep all
                    writer.write_all(record)?;
                    writer.write_all(b"\n")?;
                }
                // Else Keep nothing
                return Ok(());
            }

            // Since ranges are sorted by start, we can optimize slightly.
            // Full interval tree is overkill. Linear scan is okay for small N.
            for r in &ranges {
                if line_num >= r.start && line_num <= r.end {
                    in_range = true;
                    break;
                }
                // If line_num is less than current range start, it cannot be in any subsequent range (since sorted)
                if line_num < r.start {
                    break;
                }
            }

            let should_write = if invert {
                !in_range // Drop mode: write if NOT in range
            } else {
                in_range // Keep mode: write if IN range
            };

            if should_write {
                writer.write_all(record)?;
                writer.write_all(b"\n")?;
            }
            Ok(())
        })?;
    }

    Ok(())
}
