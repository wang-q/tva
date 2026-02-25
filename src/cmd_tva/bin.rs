use clap::*;
use std::io::{BufRead, Write};

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

    for input in crate::libs::io::input_sources(&infiles) {
        let mut reader = input.reader;
        let mut line = String::new();
        let mut first_line = true;

        while reader.read_line(&mut line)? > 0 {
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            if first_line {
                first_line = false;
                if header {
                    if !header_written {
                        // Resolve field name if needed
                        if field_idx.is_none() {
                            let headers: Vec<&str> = line.split('\t').collect();
                            if let Some(pos) =
                                headers.iter().position(|&h| h == field_str)
                            {
                                field_idx = Some(pos);
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Field '{}' not found in header",
                                    field_str
                                ));
                            }
                        }
                        if let Some(name) = new_name {
                            writeln!(writer, "{}\t{}", line, name)?;
                        } else {
                            writeln!(writer, "{}", line)?;
                        }
                        header_written = true;
                    }
                    // If header already written, skip this line (it's a header from subsequent file)
                    line.clear();
                    continue;
                }
                // No header flag, fall through to process as data
            }

            // Process data line
            let idx =
                field_idx.ok_or_else(|| anyhow::anyhow!("Field index logic error"))?;

            let fields: Vec<&str> = line.split('\t').collect();

            if idx < fields.len() {
                let val_str = fields[idx];
                let binned_val = if let Ok(val) = val_str.parse::<f64>() {
                    let binned = (val - min) / width;
                    let binned_floor = binned.floor();
                    binned_floor * width + min
                } else {
                    f64::NAN
                };

                if let Some(_) = new_name {
                    if binned_val.is_nan() {
                        writeln!(writer, "{}\t", line)?;
                    } else {
                        writeln!(writer, "{}\t{}", line, binned_val)?;
                    }
                } else {
                    // Output construction (Replace mode)
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(writer, "\t")?;
                        }
                        if i == idx && !binned_val.is_nan() {
                            write!(writer, "{}", binned_val)?;
                        } else {
                            write!(writer, "{}", field)?;
                        }
                    }
                    writeln!(writer)?;
                }
            } else {
                // Field index out of bounds
                if let Some(_) = new_name {
                    writeln!(writer, "{}\t", line)?;
                } else {
                    writeln!(writer, "{}", line)?;
                }
            }

            line.clear();
        }
    }

    Ok(())
}
