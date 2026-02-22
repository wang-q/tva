use clap::*;
use std::io::{BufRead, Write};

pub fn make_subcommand() -> Command {
    Command::new("nl")
        .about("Adds line numbers to TSV rows")
        .after_help(
            r###"
Description:
Adds a line number column to each row from one or more TSV files.

Notes:
* Supports plain text and gzipped (.gz) TSV files
* Reads from stdin if no input file is given or if input file is 'stdin'

Examples:
1. Number lines of a TSV file
   tva nl tests/genome/ctg.tsv

2. Number lines with a header for the line number column
   tva nl --header tests/genome/ctg.tsv

3. Number lines starting from 100
   tva nl --start-number 100 tests/genome/ctg.tsv
"###,
        )
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
    let mut writer = crate::libs::writer(args.get_one::<String>("outfile").unwrap());

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

    for infile in &infiles {
        let reader = crate::libs::reader(infile);
        let mut file_line_num: u64 = 0;

        for line in reader.lines().map_while(Result::ok) {
            file_line_num += 1;

            if has_header && file_line_num == 1 {
                if !header_written {
                    let header_line = if infile == "stdin" {
                        line.clone()
                    } else {
                        let mut header_reader = crate::libs::reader(infile);
                        let mut s = String::new();
                        match std::io::BufRead::read_line(&mut *header_reader, &mut s) {
                            Ok(0) | Err(_) => line.clone(),
                            Ok(_) => s.trim_end_matches(&['\n', '\r'][..]).to_string(),
                        }
                    };

                    writer.write_all(header_string.as_bytes())?;
                    writer.write_all(delimiter.as_bytes())?;
                    writer.write_all(header_line.as_bytes())?;
                    writer.write_all(b"\n")?;
                    writer.flush()?;
                    header_written = true;
                }
            } else {
                writer.write_all(line_num.to_string().as_bytes())?;
                writer.write_all(delimiter.as_bytes())?;
                writer.write_all(line.as_bytes())?;
                writer.write_all(b"\n")?;
                line_num += 1;
            }
        }
    }

    Ok(())
}
