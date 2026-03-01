use clap::*;
use std::io::{self, Write};
use std::process::{Command as ProcessCommand, Stdio};

pub fn make_subcommand() -> Command {
    Command::new("keep-header")
        .about("Executes a command while preserving the first header line")
        .after_help(include_str!("../../docs/help/keep_header.md"))
        .arg(
            Arg::new("header-lines")
                .long("lines")
                .short('n')
                .value_name("N")
                .num_args(1)
                .default_value("1")
                .value_parser(value_parser!(usize))
                .help("Number of initial header lines to preserve from the first non-empty input"),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .num_args(0..)
                .help("Input file(s)"),
        )
        .arg(
            Arg::new("command")
                .value_name("COMMAND")
                .num_args(1..)
                .last(true) // Requires -- before these args
                .required(true)
                .help("Command to run"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut header_lines: usize = *args.get_one::<usize>("header-lines").unwrap();
    if header_lines == 0 {
        header_lines = 1;
    }

    let file_args: Vec<String> = match args.get_many::<String>("files") {
        Some(values) => values.cloned().collect(),
        None => Vec::new(),
    };

    let cmd_parts: Vec<String> = match args.get_many::<String>("command") {
        Some(values) => values.cloned().collect(),
        None => {
            eprintln!("Synopsis: tva keep-header [file...] -- program [args...]");
            return Ok(());
        }
    };

    if cmd_parts.is_empty() {
        eprintln!("Synopsis: tva keep-header [file...] -- program [args...]");
        return Ok(());
    }

    let mut child = ProcessCommand::new(&cmd_parts[0])
        .args(&cmd_parts[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut child_stdin = child.stdin.take().unwrap();
    let mut header_source_used = false;

    // Use line buffering for stdout to ensure headers appear before child output where possible
    let mut stdout = io::stdout();

    let filenames: Vec<String> = if file_args.is_empty() {
        vec!["-".to_string()]
    } else {
        file_args.to_vec()
    };

    for input in crate::libs::io::raw_input_sources(&filenames) {
        let mut reader = crate::libs::tsv::reader::TsvReader::new(input.reader);

        if !header_source_used {
            let mut current_file_records = 0;

            reader.for_each_record(|line| {
                current_file_records += 1;

                if current_file_records <= header_lines {
                    stdout.write_all(line)?;
                    stdout.write_all(b"\n")?;
                    stdout.flush()?;
                } else {
                    child_stdin.write_all(line)?;
                    child_stdin.write_all(b"\n")?;
                }
                Ok(())
            })?;

            if current_file_records > 0 {
                header_source_used = true;
            }
        } else {
            let mut skipped = 0;
            reader.for_each_record(|line| {
                if skipped < header_lines {
                    skipped += 1;
                } else {
                    child_stdin.write_all(line)?;
                    child_stdin.write_all(b"\n")?;
                }
                Ok(())
            })?;
        }
    }

    drop(child_stdin);

    let status = child.wait()?;

    // Final flush not strictly needed as child has finished and we are exiting, but good practice.
    stdout.flush()?;

    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        } else {
            std::process::exit(1);
        }
    }

    Ok(())
}
