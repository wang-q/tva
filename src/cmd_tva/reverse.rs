use crate::libs::cli::{build_header_config, header_args_with_columns};
use crate::libs::tsv::header::HeaderConfig;
use clap::*;
use memchr::memrchr_iter;
use memmap2::Mmap;
use std::fs::File;
use std::io::{stdin, Read, Write};

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
        .args(header_args_with_columns())
        .arg(
            Arg::new("outfile")
                .long("outfile")
                .short('o')
                .num_args(1)
                .default_value("stdout")
                .help("Output filename. [stdout] for screen"),
        )
        .arg(
            Arg::new("no-mmap")
                .long("no-mmap")
                .action(ArgAction::SetTrue)
                .hide(true)
                .help("Disable memory mapping (for testing)"),
        )
}

/// Detect header from raw bytes according to the config.
/// Returns (header_bytes, data_start_offset).
fn detect_header_bytes(data: &[u8], config: &HeaderConfig) -> (Option<Vec<u8>>, usize) {
    if !config.enabled {
        return (None, 0);
    }

    use crate::libs::tsv::header::HeaderMode;

    match config.mode {
        HeaderMode::FirstLine => {
            // Find first newline
            if let Some(pos) = memchr::memchr(b'\n', data) {
                let header = data[..=pos].to_vec();
                (Some(header), pos + 1)
            } else if !data.is_empty() {
                // No newline but has data - treat whole thing as header
                (Some(data.to_vec()), data.len())
            } else {
                (None, 0)
            }
        }
        HeaderMode::HashLines1 => {
            // Find consecutive '#' lines, then one more line for column names
            let mut pos = 0;

            // Skip consecutive '#' lines
            while pos < data.len() {
                if data[pos] == b'#' {
                    // Find end of this line
                    if let Some(line_end) = memchr::memchr(b'\n', &data[pos..]) {
                        pos += line_end + 1;
                    } else {
                        // No newline - rest of file is a hash line
                        return (Some(data.to_vec()), data.len());
                    }
                } else {
                    break;
                }
            }

            // Now read the column names line (if we found hash lines or not)
            if pos < data.len() {
                if let Some(line_end) = memchr::memchr(b'\n', &data[pos..]) {
                    let header_end = pos + line_end + 1;
                    let header = data[..header_end].to_vec();
                    (Some(header), header_end)
                } else {
                    // No newline after column names
                    let header = data.to_vec();
                    (Some(header), data.len())
                }
            } else {
                (None, pos)
            }
        }
        _ => {
            // Other modes not expected with header_args_with_columns
            (None, 0)
        }
    }
}

fn process_buffer(
    writer: &mut dyn Write,
    data: &[u8],
    header_config: &HeaderConfig,
    header_printed: &mut bool,
) -> anyhow::Result<()> {
    // If empty, do nothing
    if data.is_empty() {
        return Ok(());
    }

    // Identify header if needed
    let mut data_start = 0;
    if header_config.enabled && !*header_printed {
        let (header_bytes, header_end) = detect_header_bytes(data, header_config);
        if let Some(header) = header_bytes {
            writer.write_all(&header)?;
            *header_printed = true;
            data_start = header_end;
        }
    }

    if data_start >= data.len() {
        return Ok(());
    }

    let slice = &data[data_start..];
    let mut following_line_start = slice.len();

    // Iterate backwards finding newlines
    for i in memrchr_iter(b'\n', slice) {
        writer.write_all(&slice[i + 1..following_line_start])?;
        following_line_start = i + 1;
    }

    // Write the first line (remainder)
    writer.write_all(&slice[0..following_line_start])?;

    Ok(())
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    // Build HeaderConfig from arguments
    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;

    let no_mmap = args.get_flag("no-mmap");
    let mut header_printed = false;

    for infile in infiles {
        if infile == "stdin" {
            let mut buf = Vec::new();
            stdin().read_to_end(&mut buf)?;
            process_buffer(&mut writer, &buf, &header_config, &mut header_printed)?;
        } else {
            let file = File::open(&infile)?;

            // Attempt mmap
            let mmap_res = if no_mmap {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "mmap disabled",
                ))
            } else {
                unsafe { Mmap::map(&file) }
            };

            match mmap_res {
                Ok(mmap) => {
                    process_buffer(
                        &mut writer,
                        &mmap,
                        &header_config,
                        &mut header_printed,
                    )?;
                }
                Err(_) => {
                    let mut f = file;
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)?;
                    process_buffer(
                        &mut writer,
                        &buf,
                        &header_config,
                        &mut header_printed,
                    )?;
                }
            }
        }
    }

    Ok(())
}
