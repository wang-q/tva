use clap::*;
use std::io::Write;

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
    let mut all_lines: Vec<Vec<u8>> = Vec::new();
    let mut header_printed = false;

    for input in crate::libs::io::raw_input_sources(&infiles) {
        if !input.is_stdin && !crate::libs::io::has_nonempty_line(&input.name)? {
            continue;
        }

        let mut reader = crate::libs::tsv::reader::TsvReader::new(input.reader);
        let mut file_line_num = 0;

        reader.for_each_record(|record| {
            file_line_num += 1;
            if header_mode && !header_printed && file_line_num == 1 {
                writer.write_all(record)?;
                writer.write_all(b"\n")?;
                header_printed = true;
            } else {
                all_lines.push(record.to_vec());
            }
            Ok(())
        })?;
    }

    // Reverse in place
    all_lines.reverse();
    
    for line in all_lines {
        writer.write_all(&line)?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}
