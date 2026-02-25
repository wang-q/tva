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

fn sanitize_field(field: &str) -> String {
    field
        .chars()
        .map(|c| match c {
            '\t' | '\n' | '\r' => ' ',
            _ => c,
        })
        .collect()
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let infile = args.get_one::<String>("infile").unwrap();
    let reader = crate::libs::io::reader(infile);
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

    let mut record = csv::StringRecord::new();

    loop {
        match csv_reader.read_record(&mut record) {
            Ok(has_record) => {
                if !has_record {
                    break;
                }

                if record.is_empty() {
                    writer.write_all(b"\n")?;
                    continue;
                }

                let mut first = true;
                for field in record.iter() {
                    if !first {
                        writer.write_all(b"\t")?;
                    }
                    let sanitized = sanitize_field(field);
                    writer.write_all(sanitized.as_bytes())?;
                    first = false;
                }
                writer.write_all(b"\n")?;
            }
            Err(err) => {
                let pos = err.position();
                if infile == "stdin" {
                    if let Some(pos) = pos {
                        let line = pos.line();
                        eprintln!("tva from csv: invalid CSV at line {}: {}", line, err);
                    } else {
                        eprintln!("tva from csv: invalid CSV: {}", err);
                    }
                } else if let Some(pos) = pos {
                    let line = pos.line();
                    eprintln!(
                        "tva from csv: invalid CSV in '{}' at line {}: {}",
                        infile, line, err
                    );
                } else {
                    eprintln!("tva from csv: invalid CSV in '{}': {}", infile, err);
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
