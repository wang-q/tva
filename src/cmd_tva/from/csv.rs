use clap::*;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("csv")
        .about("Converts CSV input to TSV")
        .after_help(include_str!("../../../docs/help/from_csv.md"))
        .arg(
            Arg::new("infile")
                .num_args(0..=1)
                .default_value("stdin")
                .index(1)
                .help("Input CSV file to process (default: stdin)"),
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
                .default_value(",")
                .help("CSV field delimiter character (default: ,)"),
        )
}

fn sanitize_field(field: &[u8], writer: &mut impl Write) -> std::io::Result<()> {
    let mut last_pos = 0;
    for (i, &byte) in field.iter().enumerate() {
        match byte {
            b'\t' | b'\n' | b'\r' => {
                if i > last_pos {
                    writer.write_all(&field[last_pos..i])?;
                }
                writer.write_all(b" ")?;
                last_pos = i + 1;
            }
            _ => {}
        }
    }
    if last_pos < field.len() {
        writer.write_all(&field[last_pos..])?;
    }
    Ok(())
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let reader = crate::libs::io::raw_reader(infile);
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .map(|s| s.as_str())
        .unwrap_or(",");
    let delimiter_bytes = delimiter_str.as_bytes();
    if delimiter_bytes.len() != 1 {
        return Err(anyhow::anyhow!(
            "delimiter must be a single byte, got \"{}\"",
            delimiter_str
        ));
    }
    let delimiter = delimiter_bytes[0];

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .from_reader(reader);

    let mut record = csv::ByteRecord::new();

    loop {
        match csv_reader.read_byte_record(&mut record) {
            Ok(true) => {
                if record.is_empty() {
                    writer.write_all(b"\n")?;
                    continue;
                }

                let mut first = true;
                for field in record.iter() {
                    if !first {
                        writer.write_all(b"\t")?;
                    }
                    sanitize_field(field, &mut writer)?;
                    first = false;
                }
                writer.write_all(b"\n")?;
            }
            Ok(false) => break,
            Err(err) => {
                let pos = err.position();
                let line_info = if let Some(pos) = pos {
                    format!(" at line {}", pos.line())
                } else {
                    String::new()
                };

                let file_info = if infile == "stdin" {
                    String::new()
                } else {
                    format!(" in '{}'", infile)
                };

                eprintln!("tva from csv: invalid CSV{}{}: {}", file_info, line_info, err);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
