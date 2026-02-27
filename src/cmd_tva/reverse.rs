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

fn process_buffer(
    writer: &mut dyn Write,
    data: &[u8],
    separator: u8,
    header_mode: bool,
    header_printed: &mut bool,
) -> anyhow::Result<()> {
    // If empty, do nothing
    if data.is_empty() {
        return Ok(());
    }

    // Identify header if needed
    let mut data_start = 0;
    if header_mode && !*header_printed {
        // Find first newline
        if let Some(pos) = memchr::memchr(separator, data) {
            let header = &data[0..=pos]; // include separator
            writer.write_all(header)?;
            *header_printed = true;
            data_start = pos + 1;
        } else {
            // Whole file is header? Or no newline.
            // If no newline, treating as header line.
            writer.write_all(data)?;
            // If data doesn't end with newline, we might want to add one if we are strict,
            // but let's just write as is.
            *header_printed = true;
            return Ok(());
        }
    }

    if data_start >= data.len() {
        return Ok(());
    }

    let slice = &data[data_start..];
    let mut following_line_start = slice.len();

    // Iterate backwards finding separators
    for i in memrchr_iter(separator, slice) {
        // Correct logic:
        // scan from right.
        // find \n.
        // This \n belongs to the line BEFORE it (in forward order).
        // The content AFTER it is the line we just finished scanning (in reverse).
        
        writer.write_all(&slice[i + 1..following_line_start])?;
        following_line_start = i + 1;
    }

    // Write the first line (remainder)
    writer.write_all(&slice[0..following_line_start])?;
    
    // Note: if the original file didn't end with \n, the first printed line (originally last) won't have \n.
    // And the last printed line (originally first) will have \n (if it had one).
    // This is consistent with tac.
    
    Ok(())
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let header_mode = args.get_flag("header");
    let mut header_printed = false;

    for infile in infiles {
        if infile == "stdin" {
            // For stdin, we must buffer it all into memory (Vec<u8>)
            // or use tempfile + mmap (like uutils tac).
            // For simplicity and speed on reasonable inputs, Vec<u8> is fine.
            // If extremely large, tempfile is better.
            // Let's use tempfile fallback if needed, or just Vec for now as starter.
            // uutils uses tempfile, let's try to be robust.
            
            let mut buf = Vec::new();
            stdin().read_to_end(&mut buf)?;
            process_buffer(&mut writer, &buf, b'\n', header_mode, &mut header_printed)?;
        } else {
            let file = File::open(&infile)?;
            // Attempt mmap
            let mmap = unsafe { Mmap::map(&file) };
            
            match mmap {
                Ok(mmap) => {
                    // We advise random access or sequential? We read backwards.
                    // madvise(MADV_SEQUENTIAL) might assume forward.
                    // backwards is not standard sequential.
                    // But usually mmap is fast enough.
                    process_buffer(&mut writer, &mmap, b'\n', header_mode, &mut header_printed)?;
                }
                Err(_) => {
                    // Fallback to reading file into Vec
                    // Or standard reading?
                    // If mmap fails (e.g. special file), read all.
                    let mut f = file; // reopen or reuse? File matches Read.
                    // File cursor is at 0.
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)?;
                    process_buffer(&mut writer, &buf, b'\n', header_mode, &mut header_printed)?;
                }
            }
        }
    }

    Ok(())
}
