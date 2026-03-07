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
                .alias("csv-delim") // Match csv2tsv --csv-delim
                .short_alias('c') // Match csv2tsv -c
                .num_args(1)
                .default_value(",")
                .help("CSV field delimiter character (default: ,)"),
        )
        .arg(
            Arg::new("quote")
                .long("quote")
                .short('q')
                .num_args(1)
                .default_value("\"")
                .help("CSV quote character (default: \")"),
        )
        .arg(
            Arg::new("tab-replacement")
                .long("tab-replacement")
                .short('r')
                .num_args(1)
                .default_value(" ")
                .help("String to replace TAB characters found in input fields"),
        )
        .arg(
            Arg::new("newline-replacement")
                .long("newline-replacement")
                .short('n')
                .num_args(1)
                .default_value(" ")
                .help("String to replace Newline characters found in input fields"),
        )
}

fn sanitize_field(
    field: &[u8],
    writer: &mut impl Write,
    tab_replacement: &[u8],
    newline_replacement: &[u8],
) -> std::io::Result<()> {
    let mut last_pos = 0;
    for (i, &byte) in field.iter().enumerate() {
        match byte {
            b'\t' => {
                if i > last_pos {
                    writer.write_all(&field[last_pos..i])?;
                }
                writer.write_all(tab_replacement)?;
                last_pos = i + 1;
            }
            b'\n' | b'\r' => {
                if i > last_pos {
                    writer.write_all(&field[last_pos..i])?;
                }
                writer.write_all(newline_replacement)?;
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
    let reader = crate::libs::io::raw_reader(infile)?;
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

    // Parse delimiter
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

    // Parse quote
    let quote_str = args
        .get_one::<String>("quote")
        .map(|s| s.as_str())
        .unwrap_or("\"");
    let quote_bytes = quote_str.as_bytes();
    if quote_bytes.len() != 1 {
        return Err(anyhow::anyhow!(
            "quote must be a single byte, got \"{}\"",
            quote_str
        ));
    }
    let quote = quote_bytes[0];

    // Parse replacements
    let tab_replacement = args
        .get_one::<String>("tab-replacement")
        .map(|s| s.as_bytes())
        .unwrap_or(b" ");
    let newline_replacement = args
        .get_one::<String>("newline-replacement")
        .map(|s| s.as_bytes())
        .unwrap_or(b" ");

    // Validation: Delimiter and Quote cannot be newline
    if delimiter == b'\n' || delimiter == b'\r' {
        return Err(anyhow::anyhow!(
            "CSV field delimiter cannot be newline (--d|delimiter|csv-delim)."
        ));
    }
    if quote == b'\n' || quote == b'\r' {
        return Err(anyhow::anyhow!(
            "CSV quote character cannot be newline (--q|quote)."
        ));
    }
    if delimiter == quote {
        return Err(anyhow::anyhow!(
            "CSV quote and CSV field delimiter characters must be different (--q|quote, --d|delimiter)."
        ));
    }
    // Validation: Replacements cannot contain delimiter or newline (though csv2tsv says 'TSV delimiter', which is TAB)
    // csv2tsv logic: "Replacement character cannot contain newlines or TSV field delimiters"
    // TSV delimiter is TAB (b'\t').
    if tab_replacement.contains(&b'\t')
        || tab_replacement.contains(&b'\n')
        || tab_replacement.contains(&b'\r')
    {
        return Err(anyhow::anyhow!(
            "Replacement character cannot contain newlines or TSV field delimiters (--r|tab-replacement)."
        ));
    }
    if newline_replacement.contains(&b'\t')
        || newline_replacement.contains(&b'\n')
        || newline_replacement.contains(&b'\r')
    {
        return Err(anyhow::anyhow!(
            "Replacement character cannot contain newlines or TSV field delimiters (--n|newline-replacement)."
        ));
    }

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .quote(quote)
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

                // Rewrite loop to use writer directly
                let mut first = true;
                for field in record.iter() {
                    if !first {
                        writer.write_all(b"\t")?;
                    }
                    sanitize_field(
                        field,
                        &mut writer,
                        tab_replacement,
                        newline_replacement,
                    )?;
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

                // Match csv2tsv error format slightly better if needed, but this is fine.
                // csv2tsv: "Error [csv2tsv]: Invalid CSV. Improperly terminated quoted field. File: invalid1.csv, Line: 3"
                eprintln!(
                    "tva from csv: invalid CSV{}{}: {}",
                    file_info, line_info, err
                );
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
