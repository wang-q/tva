use clap::*;
use std::io::{self, BufRead, Write};
use std::process::{Command as ProcessCommand, Stdio};

pub fn make_subcommand() -> Command {
    Command::new("keep-header")
        .about("Executes a command while preserving the first header line")
        .after_help(
            r###"
Runs an external command in a header-aware fashion. The first line of each
input file is treated as a header. The first header line is written to standard
output unchanged. All remaining lines (from all files) are sent to the given
command via standard input, excluding header lines from subsequent files. The
output produced by the command is appended after the initial header line.

Usage:
  tva keep-header [file...] -- program [args...]

 Notes:
 - If no input files are given, data is read from standard input.
 - The number of header lines to preserve from the first non-empty input can be
   configured with --lines / -n (default: 1).
- A double dash (--) separates input files from the command to run, similar
  to how the pipe operator (|) separates commands in a shell pipeline.
- The command is run with its standard input connected to the concatenated
  data lines (all lines after the first header line of each file).
- The command's standard output and standard error are passed through to
  this process.

Examples:
1. Sort a file while keeping the header line first
   tva keep-header data.tsv -- sort

2. Sort multiple TSV files numerically on field 2, preserving one header
   tva keep-header data1.tsv data2.tsv -- sort -t $'\t' -k2,2n

3. Read from stdin, filter with grep, and keep the original header
   cat data.tsv | tva keep-header -- grep red
"###,
        )
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
            Arg::new("args")
                .value_name("ARGS")
                .num_args(1..)
                .trailing_var_arg(true)
                .help("Usage: tva keep-header [file...] -- program [args...]"),
        )
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut header_lines: usize = *args.get_one::<usize>("header-lines").unwrap();
    if header_lines == 0 {
        header_lines = 1;
    }

    let raw_args: Vec<String> = match args.get_many::<String>("args") {
        Some(values) => values.cloned().collect(),
        None => Vec::new(),
    };

    let split_pos = raw_args.iter().position(|s| s == "--");

    if split_pos.is_none() || split_pos == Some(raw_args.len() - 1) {
        eprintln!("Synopsis: tva keep-header [file...] -- program [args...]");
        return Ok(());
    }

    let split_index = split_pos.unwrap();
    let file_args = &raw_args[..split_index];
    let cmd_parts = &raw_args[split_index + 1..];

    let mut child = ProcessCommand::new(&cmd_parts[0])
        .args(&cmd_parts[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut child_stdin = child.stdin.take().unwrap();
    let mut header_source_used = false;
    let mut stdout = io::stdout();

    let filenames: Vec<String> = if file_args.is_empty() {
        vec!["-".to_string()]
    } else {
        file_args.to_vec()
    };

    for name in filenames {
        let mut file_had_content = false;
        let mut remaining = header_lines;

        let mut reader = crate::libs::io::reader(&name);
        let mut line = String::new();
        loop {
            line.clear();
            let n = BufRead::read_line(&mut *reader, &mut line)?;
            if n == 0 {
                break;
            }
            file_had_content = true;
            if remaining > 0 {
                if !header_source_used {
                    stdout.write_all(line.as_bytes())?;
                }
                remaining -= 1;
            } else {
                child_stdin.write_all(line.as_bytes())?;
            }
        }

        if file_had_content && !header_source_used {
            stdout.flush()?;
            header_source_used = true;
        }

        child_stdin.flush()?;
    }

    drop(child_stdin);
    let _status = child.wait()?;

    Ok(())
}
