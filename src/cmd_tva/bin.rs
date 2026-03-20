use clap::*;
use std::io::Write;

use crate::libs::cli::{
    build_header_config, delimiter_arg, get_delimiter, header_args_with_columns,
};
use crate::libs::io::map_io_err;

use crate::libs::tsv::header::Header;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::{Row, TsvRow};

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
                .help("Bin alignment origin (bin edges align to min + n*width)"),
        )
        .arg(
            Arg::new("new-name")
                .long("new-name")
                .num_args(1)
                .help("Append as new column with this name (instead of replacing)"),
        )
        .args(header_args_with_columns())
        .arg(delimiter_arg())
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
    let mut writer =
        crate::libs::io::writer(args.get_one::<String>("outfile").unwrap())?;

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
    let new_name = args.get_one::<String>("new-name");

    let header_config =
        build_header_config(args, true).map_err(|e| anyhow::anyhow!(e))?;
    let has_header = header_config.enabled;
    let opt_delimiter = get_delimiter(args, "delimiter")?;

    let mut header_written = false;
    let mut field_idx: Option<usize> = None;

    // Pre-check: if field is numeric, we can parse it now
    if let Ok(idx) = field_str.parse::<usize>() {
        if idx == 0 {
            return Err(anyhow::anyhow!("Field index must be >= 1"));
        }
        field_idx = Some(idx - 1);
    }

    for input in crate::libs::io::raw_input_sources(&infiles)? {
        let mut tsv_reader = TsvReader::with_capacity(input.reader, 512 * 1024);

        if has_header {
            if !header_written {
                let header_result = tsv_reader
                    .read_header_mode(header_config.mode)
                    .map_err(map_io_err)?;

                let header_info = match header_result {
                    Some(info) => info,
                    None => continue, // Empty file, skip to next
                };

                // Note: column_names_line is always Some for FirstLine and HashLines1 modes
                // (the only modes supported by bin command)
                let column_names_bytes = header_info.column_names_line.unwrap();

                if field_idx.is_none() {
                    let h = Header::from_column_names(
                        column_names_bytes.clone(),
                        opt_delimiter as char,
                    );
                    if let Some(pos) = h.get_index(field_str) {
                        field_idx = Some(pos);
                    } else {
                        return Err(anyhow::anyhow!(
                            "Field '{}' not found in header",
                            field_str
                        ));
                    }
                }

                writer.write_all(&column_names_bytes)?;
                if let Some(name) = new_name {
                    writer.write_all(b"\t")?;
                    writer.write_all(name.as_bytes())?;
                }
                writer.write_all(b"\n")?;
                header_written = true;
            } else {
                // For subsequent files, skip the header
                let _ = tsv_reader
                    .read_header_mode(header_config.mode)
                    .map_err(map_io_err)?;
            }
        } else if field_idx.is_none() {
            return Err(anyhow::anyhow!(
                "Field name '{}' requires --header",
                field_str
            ));
        }

        tsv_reader.for_each_row(opt_delimiter, |row: &TsvRow| {
            // SAFETY: field_idx is always Some here (validated earlier)
            let idx = field_idx.unwrap();
            let num_fields = row.field_count();

            if new_name.is_some() {
                writer.write_all(row.line)?;
                writer.write_all(b"\t")?;

                // Get field at idx (0-based)
                if let Some(bytes) = row.get_bytes(idx + 1) {
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
                for col_idx in 0..num_fields {
                    if col_idx > 0 {
                        writer.write_all(b"\t")?;
                    }

                    let bytes = row.get_bytes(col_idx + 1).unwrap_or(b"");

                    if col_idx == idx {
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
                        writer.write_all(bytes)?;
                    }
                }
                writer.write_all(b"\n")?;
            }

            Ok(())
        })?;
    }

    Ok(())
}
