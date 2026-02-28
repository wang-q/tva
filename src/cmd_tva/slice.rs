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
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Always preserve the first line (header)"),
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
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let invert = args.get_flag("invert");
    let keep_header = args.get_flag("header");

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

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);
        let mut line_num = 0;

        reader.for_each_record(|record| {
            line_num += 1;

            // Always keep header if requested
            if line_num == 1 && keep_header {
                writer.write_all(record)?;
                writer.write_all(b"\n")?;
                return Ok(());
            }

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
            // But full interval tree is overkill. Linear scan is okay for small N.
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
