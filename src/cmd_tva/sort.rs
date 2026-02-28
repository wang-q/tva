use clap::*;
use crate::libs::tsv::record::TsvRecord;
use crate::libs::key::{KeyExtractor, KeyBuffer};
use intspan::IntSpan;
use std::cmp::Ordering;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("sort")
        .about("Sorts TSV/CSV records by one or more keys")
        .after_help(include_str!("../../docs/help/sort.md"))
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input file(s) to sort (default: stdin)"),
        )
        .arg(
            Arg::new("key")
                .long("key")
                .short('k')
                .num_args(1)
                .help("Field list (1-based) to use as sort key, e.g. 2 or 2,4-5"),
        )
        .arg(
            Arg::new("numeric")
                .long("numeric")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Compare key fields numerically"),
        )
        .arg(
            Arg::new("reverse")
                .long("reverse")
                .short('r')
                .action(ArgAction::SetTrue)
                .help("Reverse the sort order"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line of each file as a header; only the first header is output"),
        )
        .arg(
            Arg::new("delimiter")
                .long("delimiter")
                .short('t')
                .num_args(1)
                .default_value("\t")
                .help("Field delimiter character (default: TAB)"),
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
    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let numeric = args.get_flag("numeric");
    let reverse = args.get_flag("reverse");
    let has_header = args.get_flag("header");

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .map(|s| s.as_str())
        .unwrap_or("\t");
    let delimiter_bytes = delimiter_str.as_bytes();
    if delimiter_bytes.len() != 1 {
        return Err(anyhow::anyhow!(
            "delimiter must be a single byte, got \"{}\"",
            delimiter_str
        ));
    }
    let delimiter = delimiter_bytes[0];

    let key_indices: Vec<usize> = if let Some(spec) = args.get_one::<String>("key") {
        match parse_key_indices(spec) {
            Ok(v) => v,
            Err(msg) => {
                return Err(anyhow::anyhow!(
                    "invalid key specification `{}`: {}",
                    spec,
                    msg
                ));
            }
        }
    } else {
        Vec::new()
    };

    let mut rows: Vec<TsvRecord> = Vec::new();
    let mut header_record: Option<TsvRecord> = None;
    let mut header_written = false;

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut reader = crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);
        let mut is_first_line = true;

        reader.for_each_record(|record| {
            if has_header && is_first_line {
                if header_record.is_none() {
                    let mut tsv_rec = TsvRecord::new();
                    tsv_rec.parse_line(record, delimiter);
                    header_record = Some(tsv_rec);
                }
                is_first_line = false;
                return Ok(());
            }

            if record.is_empty() {
                rows.push(TsvRecord::new());
            } else {
                let mut tsv_rec = TsvRecord::new();
                tsv_rec.parse_line(record, delimiter);
                rows.push(tsv_rec);
            }
            
            is_first_line = false;
            Ok(())
        })?;
    }

    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    if let Some(header) = header_record {
        writer.write_all(header.as_line())?;
        writer.write_all(b"\n")?;
        header_written = true;
    }

    if rows.is_empty() {
        if !header_written {
             writer.flush()?;
        }
        return Ok(());
    }

    let indices: Vec<usize> = if key_indices.is_empty() {
        let max_fields = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        (1..=max_fields).collect() // 1-based indices for all fields
    } else {
        key_indices.iter().map(|&i| i + 1).collect() // Convert 0-based to 1-based
    };

    if numeric {
        // Pre-compute numeric keys
        let mut keyed_rows: Vec<(Vec<f64>, TsvRecord)> = Vec::with_capacity(rows.len());
        for record in rows {
            let mut key = Vec::with_capacity(indices.len());
            for &idx in &indices { // idx is 1-based
                let field = record.get(idx - 1).unwrap_or(b""); // get is 0-based
                // Optimization: parse directly from bytes without full utf8 check if possible,
                // but standard parse requires &str.
                // from_utf8_lossy might allocate, from_utf8 is better.
                let s = std::str::from_utf8(field).unwrap_or("");
                let n: f64 = s.trim().parse().unwrap_or(0.0);
                key.push(n);
            }
            keyed_rows.push((key, record));
        }

        keyed_rows.sort_by(|(ka, _), (kb, _)| {
            let mut ord = Ordering::Equal;
            for (na, nb) in ka.iter().zip(kb.iter()) {
                ord = na.partial_cmp(nb).unwrap_or(Ordering::Equal);
                if ord != Ordering::Equal {
                    break;
                }
            }
            if reverse {
                ord.reverse()
            } else {
                ord
            }
        });

        for (_, record) in keyed_rows {
            if record.is_empty() {
                writer.write_all(b"\n")?;
            } else {
                writer.write_all(record.as_line())?;
                writer.write_all(b"\n")?;
            }
        }
    } else {
        // Pre-compute byte keys using KeyExtractor
        // Note: Sort behavior for multiple keys:
        // KeyExtractor concatenates keys with delimiter.
        // "A\tB" vs "A\tC".
        // If delimiter < any content char, this is equivalent to tuple comparison.
        // Tab is \t (9). Visible chars >= 32.
        // So this optimization is valid for standard text.
        
        // KeyExtractor now expects 1-based indices.
        let mut extractor = KeyExtractor::new(Some(indices), false, false); // strict=false to handle missing fields gracefully
        
        // We use KeyBuffer (SmallVec) to store keys.
        let mut keyed_rows: Vec<(KeyBuffer, TsvRecord)> = Vec::with_capacity(rows.len());
        
        for record in rows {
            let key_res = extractor.extract_from_record(&record, delimiter);
            let key = match key_res {
                Ok(k) => k.into_owned(),
                Err(_) => KeyBuffer::new(), // Should not happen with strict=false
            };
            keyed_rows.push((key, record));
        }

        if reverse {
            keyed_rows.sort_by(|(ka, _), (kb, _)| kb.cmp(ka));
        } else {
            keyed_rows.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
        }

        for (_, record) in keyed_rows {
            if record.is_empty() {
                writer.write_all(b"\n")?;
            } else {
                writer.write_all(record.as_line())?;
                writer.write_all(b"\n")?;
            }
        }
    }

    Ok(())
}

fn parse_key_indices(spec: &str) -> Result<Vec<usize>, String> {
    if spec.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut indices: Vec<usize> = Vec::new();
    let mut seen: std::collections::HashSet<i32> = std::collections::HashSet::new();

    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(format!("empty key list element in `{}`", spec));
        }
        let span = IntSpan::from(part);
        for e in span.elements() {
            if e <= 0 {
                return Err(format!("field index must be >= 1 in `{}`", spec));
            }
            if seen.insert(e) {
                indices.push((e - 1) as usize);
            }
        }
    }

    Ok(indices)
}
