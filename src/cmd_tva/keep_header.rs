use clap::*;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command as ProcessCommand, Stdio};

const COPY_BUFFER_SIZE: usize = 128 * 1024; // 128KB

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
                .last(true)
                .required(true)
                .help("Command to run"),
        )
}

fn copy_with_buffer<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    buffer: &mut [u8],
) -> io::Result<u64> {
    let mut total = 0u64;
    loop {
        let n = reader.read(buffer)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buffer[..n])?;
        total += n as u64;
    }
    Ok(total)
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
    let mut stdout = io::stdout();
    let mut copy_buffer = vec![0u8; COPY_BUFFER_SIZE];

    let filenames: Vec<String> = if file_args.is_empty() {
        vec!["-".to_string()]
    } else {
        file_args.to_vec()
    };

    for input in crate::libs::io::raw_input_sources(&filenames)? {
        let reader = input.reader;

        if !header_source_used {
            // First file: extract header lines, then stream the rest
            let mut buf_reader = BufReader::new(reader);
            let mut current_line = 0;

            // Read and output header lines
            loop {
                if current_line >= header_lines {
                    break;
                }

                let mut line = Vec::new();
                let bytes_read = buf_reader.read_until(b'\n', &mut line)?;

                if bytes_read == 0 {
                    // EOF before reading all header lines
                    break;
                }

                // Output header to stdout
                stdout.write_all(&line)?;
                stdout.flush()?;
                current_line += 1;
            }

            // Stream remaining data directly to child stdin using large buffer
            copy_with_buffer(&mut buf_reader, &mut child_stdin, &mut copy_buffer)?;

            if current_line > 0 {
                header_source_used = true;
            }
        } else {
            // Subsequent files: skip header lines, stream the rest
            let mut buf_reader = BufReader::new(reader);
            let mut skipped = 0;

            // Skip header lines
            loop {
                if skipped >= header_lines {
                    break;
                }

                let mut line = Vec::new();
                let bytes_read = buf_reader.read_until(b'\n', &mut line)?;

                if bytes_read == 0 {
                    // EOF before skipping all header lines
                    break;
                }

                skipped += 1;
            }

            // Stream remaining data directly to child stdin using large buffer
            copy_with_buffer(&mut buf_reader, &mut child_stdin, &mut copy_buffer)?;
        }
    }

    drop(child_stdin);

    let status = child.wait()?;
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
