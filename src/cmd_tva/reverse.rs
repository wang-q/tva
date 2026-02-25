use clap::*;
use std::io::{BufRead, Write};

pub fn make_subcommand() -> Command {
    Command::new("reverse")
        .about("Reverses the order of lines (like tac)")
        .after_help(include_str!("../../docs/help/reverse.md"))
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
                .help("Treat the first line as a header and print it first"),
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
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let header_mode = args.get_flag("header");
    let mut all_lines: Vec<String> = Vec::new();
    let mut header_printed = false;

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;
        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            if header_mode && !header_printed && i == 0 {
                writeln!(writer, "{}", line)?;
                header_printed = true;
            } else {
                all_lines.push(line);
            }
        }
    }

    all_lines.reverse();
    for line in all_lines {
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}
