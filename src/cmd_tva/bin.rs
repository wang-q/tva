use crate::libs::tsv::fields::Header;
use crate::libs::tsv::reader::TsvReader;
use clap::*;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("bin")
        .about("Discretize numeric values into bins (useful for histograms)")
        .after_help(include_str!("../../docs/help/bin.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("width")
                .long("width")
                .short('w')
                .required(true)
                .value_parser(value_parser!(f64))
                .help("Bin width (bucket size)"),
        )
        .arg(
            Arg::new("field")
                .long("field")
                .short('f')
                .required(true)
                .help("Field to bin (1-based index or name)"),
        )
        .arg(
            Arg::new("min")
                .long("min")
                .short('m')
                .default_value("0.0")
                .value_parser(value_parser!(f64))
                .help("Alignment/Offset (bin start)"),
        )
        .arg(
            Arg::new("new-name")
                .long("new-name")
                .num_args(1)
                .help("Append as new column with this name (instead of replacing)"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Input has header"),
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

    let width = *args.get_one::<f64>("width").unwrap();
    if width <= 0.0 {
        return Err(anyhow::anyhow!("Width must be positive"));
    }
    let min = *args.get_one::<f64>("min").unwrap();
    let field_str = args.get_one::<String>("field").unwrap();
    let header = args.get_flag("header");
    let new_name = args.get_one::<String>("new-name");

    // Pre-check: if field is not numeric, header is required
    let is_numeric_field = field_str.chars().all(|c| c.is_ascii_digit());
    if !is_numeric_field && !header {
        return Err(anyhow::anyhow!(
            "Field name '{}' requires --header",
            field_str
        ));
    }

    let mut header_written = false;
    let mut field_idx: Option<usize> = None;
    // If field is numeric, we can parse it now
    if let Ok(idx) = field_str.parse::<usize>() {
        if idx == 0 {
            return Err(anyhow::anyhow!("Field index must be >= 1"));
        }
        field_idx = Some(idx - 1);
    }

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut reader = TsvReader::new(input.reader);
        let mut is_first_line = true;

        reader.for_each_record(|record| {
            if is_first_line {
                is_first_line = false;
                if header {
                    if !header_written {
                        // Resolve field name if needed
                        if field_idx.is_none() {
                            let line_str = std::str::from_utf8(record).map_err(|e| {
                                std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                            })?;
                            let h = Header::from_line(line_str, '\t');
                            if let Some(pos) = h.get_index(field_str) {
                                field_idx = Some(pos);
                            } else {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!("Field '{}' not found in header", field_str),
                                ));
                            }
                        }

                        writer.write_all(record)?;
                        if let Some(name) = new_name {
                            writer.write_all(b"\t")?;
                            writer.write_all(name.as_bytes())?;
                        }
                        writer.write_all(b"\n")?;
                        header_written = true;
                    }
                    // If header already written, skip this line (it's a header from subsequent file)
                    return Ok(());
                }
                // No header flag, fall through to process as data
            }

            // Process data line
            let idx = field_idx.ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Field index logic error")
            })?;

            if new_name.is_some() {
                writer.write_all(record)?;
                writer.write_all(b"\t")?;

                // Find the field at idx to calculate value
                // We optimize by just scanning tabs
                let mut iter = memchr::memchr_iter(b'\t', record);
                let mut field_bytes = None;

                if idx == 0 {
                    let end = iter.next().unwrap_or(record.len());
                    field_bytes = Some(&record[0..end]);
                } else {
                    // Skip idx-1 tabs
                    let mut skipped = 0;
                    for _ in 0..idx - 1 {
                        if iter.next().is_some() {
                            skipped += 1;
                        } else {
                            break;
                        }
                    }
                    if skipped == idx - 1 {
                        if let Some(start_pos) = iter.next() {
                            let start = start_pos + 1;
                            let end = iter.next().unwrap_or(record.len());
                            field_bytes = Some(&record[start..end]);
                        }
                    }
                }

                if let Some(bytes) = field_bytes {
                    if let Ok(s) = std::str::from_utf8(bytes) {
                        if let Ok(val) = s.parse::<f64>() {
                            let binned = (val - min) / width;
                            let binned_floor = binned.floor();
                            let result = binned_floor * width + min;
                            write!(writer, "{}", result)?;
                        }
                    }
                }
                writer.write_all(b"\n")?;
            } else {
                // Replace mode
                let mut last_pos = 0;
                let mut current_col = 0;
                let mut iter = memchr::memchr_iter(b'\t', record);

                loop {
                    let (end_pos, is_last) = match iter.next() {
                        Some(pos) => (pos, false),
                        None => (record.len(), true),
                    };

                    if current_col > 0 {
                        writer.write_all(b"\t")?;
                    }

                    if current_col == idx {
                        let bytes = &record[last_pos..end_pos];
                        let mut written = false;
                        if let Ok(s) = std::str::from_utf8(bytes) {
                            if let Ok(val) = s.parse::<f64>() {
                                let binned = (val - min) / width;
                                let binned_floor = binned.floor();
                                let result = binned_floor * width + min;
                                write!(writer, "{}", result)?;
                                written = true;
                            }
                        }
                        if !written {
                            writer.write_all(bytes)?;
                        }
                    } else {
                        writer.write_all(&record[last_pos..end_pos])?;
                    }

                    if is_last {
                        break;
                    }
                    last_pos = end_pos + 1;
                    current_col += 1;
                }
                writer.write_all(b"\n")?;
            }

            Ok(())
        })?;
    }

    Ok(())
}
