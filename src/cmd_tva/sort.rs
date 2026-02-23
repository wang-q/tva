use clap::*;
use intspan::IntSpan;
use std::cmp::Ordering;
use std::io::BufRead;

pub fn make_subcommand() -> Command {
    Command::new("sort")
        .about("Sorts TSV/CSV records by one or more keys")
        .after_help(
            r###"
Sorts tab-separated values (TSV) or other delimited text by one or more key fields.

Input:
- If no input files are given, or an input file is 'stdin', data is read
  from standard input.
- Files ending in '.gz' are transparently decompressed.

Keys:
- Use -k/--key to specify 1-based field indices or ranges (for example: 2,4-5).
- Multiple keys are supported and are applied in the order given.

Behavior:
- By default, comparisons are lexicographic.
- With -n/--numeric, comparisons are numeric (floating point).
- With -r/--reverse, the final ordering is reversed.
- For an MxN table, the output contains the same rows sorted by the selected key fields.
- Empty fields compare as empty strings in lexicographic mode and as 0 in numeric mode.

Output:
- Writes sorted records to standard output or to the file given by --outfile.
"###,
        )
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
    let delimiter = delimiter_bytes[0] as char;

    let key_indices: Vec<usize> = if let Some(spec) = args.get_one::<String>("key") {
        match parse_key_indices(spec) {
            Ok(v) => v,
            Err(msg) => {
                return Err(anyhow::anyhow!("invalid key specification `{}`: {}", spec, msg));
            }
        }
    } else {
        Vec::new()
    };

    let mut rows: Vec<Vec<String>> = Vec::new();

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;

        for line in reader.lines().map_while(Result::ok) {
            if line.is_empty() {
                rows.push(Vec::new());
            } else {
                let fields: Vec<String> = line
                    .split(delimiter)
                    .map(|s| s.to_string())
                    .collect();
                rows.push(fields);
            }
        }
    }

    if rows.is_empty() {
        let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());
        writer.flush()?;
        return Ok(());
    }

    let indices = if key_indices.is_empty() {
        let max_fields = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        (0..max_fields).collect()
    } else {
        key_indices
    };

    rows.sort_by(|a, b| compare_rows(a, b, &indices, numeric, reverse));

    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());
    let delim_out = delimiter.to_string();

    for fields in rows {
        if fields.is_empty() {
            writer.write_all(b"\n")?;
        } else {
            let line = fields.join(&delim_out);
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    Ok(())
}

fn compare_rows(
    a: &Vec<String>,
    b: &Vec<String>,
    indices: &[usize],
    numeric: bool,
    reverse: bool,
) -> Ordering {
    let mut ord = Ordering::Equal;

    for &idx in indices {
        let av = a.get(idx).map(|s| s.as_str()).unwrap_or("");
        let bv = b.get(idx).map(|s| s.as_str()).unwrap_or("");

        let key_ord = if numeric {
            let an: f64 = av.parse().unwrap_or(0.0);
            let bn: f64 = bv.parse().unwrap_or(0.0);
            an.partial_cmp(&bn).unwrap_or(Ordering::Equal)
        } else {
            av.cmp(bv)
        };

        if key_ord != Ordering::Equal {
            ord = key_ord;
            break;
        }
    }

    if reverse {
        ord.reverse()
    } else {
        ord
    }
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
