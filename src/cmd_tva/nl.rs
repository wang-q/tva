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
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line of each file as a header; only the first header is output"),
        )
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
                .help("Enable line-buffered output (for compatibility; behaves like default buffering)"),
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

    let mut has_header = args.get_flag("header");
    let header_string = if let Some(s) = args.get_one::<String>("header-string") {
        has_header = true;
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
        let mut file_line_num: u64 = 0;

        reader.for_each_record(|line| {
            file_line_num += 1;

            if has_header && file_line_num == 1 {
                if !header_written {
                    writer.write_all(header_bytes)?;
                    writer.write_all(delimiter_bytes)?;
                    writer.write_all(line)?;
                    writer.write_all(b"\n")?;
                    writer.flush()?;
                    header_written = true;
                }
            } else {
                write!(writer, "{}", line_num)?;
                writer.write_all(delimiter_bytes)?;
                writer.write_all(line)?;
                writer.write_all(b"\n")?;
                if line_buffered {
                    writer.flush()?;
                }
                line_num += 1;
            }
            Ok(())
        })?;
    }

    Ok(())
}
