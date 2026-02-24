use crate::libs::fields;
use crate::libs::fields::Header;
use clap::*;
use indexmap::{IndexMap, IndexSet};
use std::io::{BufRead, Write};

pub fn make_subcommand() -> Command {
    Command::new("wider")
        .about("Reshapes a long table into a wide format")
        .after_help(
            r###"
Reshapes a table from long to wide format by spreading a key-value pair across
multiple columns. This is the inverse of 'longer'.

Input:
- Reads from one or more TSV files or standard input.
- Files ending in '.gz' are transparently decompressed.
- The first line is ALWAYS treated as a header.
- When multiple files are provided, they must have the SAME column structure.

Reshaping behavior:
- --names-from
  Column(s) containing the new column headers.
- --values-from
  Column(s) containing the data values.
- --id-cols
  Columns that identify each row. If omitted, all columns except
  'names-from' and 'values-from' are used.
- --values-fill
  Value to use for missing cells (default: empty).
- --names-sort
  Sort the resulting column headers alphabetically.

Examples:
1. Spread 'key' and 'value' columns back into wide format
   tva wider data.tsv --names-from key --values-from value

2. Spread 'measurement' column, using 'result' as values
   tva wider data.tsv --names-from measurement --values-from result

3. Specify ID columns explicitly (dropping others)
   tva wider data.tsv --names-from key --values-from val --id-cols id date
"###,
        )
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("names-from")
                .long("names-from")
                .required(true)
                .help("Column(s) containing the new column headers"),
        )
        .arg(
            Arg::new("values-from")
                .long("values-from")
                .required(true)
                .help("Column(s) containing the data values"),
        )
        .arg(
            Arg::new("id-cols")
                .long("id-cols")
                .help("Columns that identify each row (default: all others)"),
        )
        .arg(
            Arg::new("values-fill")
                .long("values-fill")
                .default_value("")
                .help("Value to use for missing cells"),
        )
        .arg(
            Arg::new("names-sort")
                .long("names-sort")
                .action(ArgAction::SetTrue)
                .help("Sort the resulting column headers"),
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

struct WiderConfig {
    names_from: String,
    values_from: String,
    id_cols: Option<String>,
    fill_value: String,
    sort_names: bool,
}

struct ProcessState {
    // Key: ID columns values
    // Value: Map of Name -> Value
    data: IndexMap<Vec<String>, IndexMap<String, String>>,
    all_names: IndexSet<String>,

    // Indices (0-based)
    names_idx: usize,
    values_idx: usize,
    id_indices: Vec<usize>,

    // Header info
    header_processed: bool,
    output_header_prefix: Vec<String>,
    first_file_header_len: usize,
}

impl ProcessState {
    fn new() -> Self {
        Self {
            data: IndexMap::new(),
            all_names: IndexSet::new(),
            names_idx: 0,
            values_idx: 0,
            id_indices: Vec::new(),
            header_processed: false,
            output_header_prefix: Vec::new(),
            first_file_header_len: 0,
        }
    }
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let config = WiderConfig {
        names_from: args.get_one::<String>("names-from").unwrap().clone(),
        values_from: args.get_one::<String>("values-from").unwrap().clone(),
        id_cols: args.get_one::<String>("id-cols").cloned(),
        fill_value: args.get_one::<String>("values-fill").unwrap().clone(),
        sort_names: args.get_flag("names-sort"),
    };

    let mut state = ProcessState::new();

    for infile in &infiles {
        process_file(infile, &config, &mut state)?;
    }

    write_output(&mut writer, &state, &config)?;

    Ok(())
}

fn process_file(
    infile: &str,
    config: &WiderConfig,
    state: &mut ProcessState,
) -> anyhow::Result<()> {
    let mut reader = crate::libs::io::reader(infile);

    // Read header
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(());
    }
    trim_newline(&mut line);

    let header_fields: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();
    let header = Header::from_fields(header_fields.clone());

    if !state.header_processed {
        // Determine indices
        let n_indices = fields::parse_field_list_with_header(
            &config.names_from,
            Some(&header),
            '\t',
        )
        .map_err(|e| anyhow::anyhow!(e))?;
        if n_indices.len() != 1 {
            return Err(anyhow::anyhow!(
                "Currently only single column supported for --names-from"
            ));
        }
        state.names_idx = n_indices[0] - 1;

        let v_indices = fields::parse_field_list_with_header(
            &config.values_from,
            Some(&header),
            '\t',
        )
        .map_err(|e| anyhow::anyhow!(e))?;
        if v_indices.len() != 1 {
            return Err(anyhow::anyhow!(
                "Currently only single column supported for --values-from"
            ));
        }
        state.values_idx = v_indices[0] - 1;

        if let Some(spec) = &config.id_cols {
            let i_indices =
                fields::parse_field_list_with_header(spec, Some(&header), '\t')
                    .map_err(|e| anyhow::anyhow!(e))?;
            state.id_indices = i_indices.iter().map(|&i| i - 1).collect();
        } else {
            // Default: all except names and values
            for (i, _) in header_fields.iter().enumerate() {
                if i != state.names_idx && i != state.values_idx {
                    state.id_indices.push(i);
                }
            }
        }

        // Store output header prefix (ID column names)
        for &i in &state.id_indices {
            state.output_header_prefix.push(header_fields[i].clone());
        }

        state.first_file_header_len = header_fields.len();
        state.header_processed = true;
    } else {
        // Validate subsequent file headers
        if header_fields.len() != state.first_file_header_len {
            return Err(anyhow::anyhow!(
                 "File '{}' has {} columns, but first file had {}. All files must have the same column structure.",
                 infile, header_fields.len(), state.first_file_header_len
             ));
        }
        // Ideally we should also check column names, but for now length check is a basic safeguard.
    }

    // Process rows
    for line in reader.lines() {
        let mut line = line?;
        trim_newline(&mut line);

        let fields: Vec<&str> = line.split('\t').collect();

        // Validate fields length against indices
        let max_idx = std::cmp::max(state.names_idx, state.values_idx);
        if max_idx >= fields.len() {
            // Skip malformed lines or error?
            // To be safe, we can skip or error. Let's error to be strict.
            // But actually, split('\t') on "A" gives ["A"]. If we need index 1, it fails.
            // Let's just fill with empty string if missing, to be consistent with previous logic,
            // BUT previous logic had a bug with trim_end().
            // If the file is valid TSV, it should have enough columns.
        }

        // Extract ID key
        let mut key: Vec<String> = Vec::with_capacity(state.id_indices.len());
        for &i in &state.id_indices {
            if i < fields.len() {
                key.push(fields[i].to_string());
            } else {
                key.push("".to_string());
            }
        }

        // Extract Name
        let name = if state.names_idx < fields.len() {
            fields[state.names_idx].to_string()
        } else {
            "".to_string()
        };

        // Extract Value
        let value = if state.values_idx < fields.len() {
            fields[state.values_idx].to_string()
        } else {
            "".to_string()
        };

        state.all_names.insert(name.clone());

        state
            .data
            .entry(key)
            .or_insert_with(IndexMap::new)
            .insert(name, value);
    }

    Ok(())
}

fn write_output<W: Write>(
    writer: &mut W,
    state: &ProcessState,
    config: &WiderConfig,
) -> anyhow::Result<()> {
    // Sort names if requested
    let final_names: Vec<String> = if config.sort_names {
        let mut sorted: Vec<String> = state.all_names.iter().cloned().collect();
        sorted.sort();
        sorted
    } else {
        state.all_names.iter().cloned().collect()
    };

    // Write Header
    for (i, col) in state.output_header_prefix.iter().enumerate() {
        if i > 0 {
            write!(writer, "\t")?;
        }
        write!(writer, "{}", col)?;
    }
    for name in &final_names {
        if !state.output_header_prefix.is_empty() {
            write!(writer, "\t")?;
        }
        write!(writer, "{}", name)?;
    }
    writeln!(writer)?;

    // Write Rows
    for (key, row_map) in &state.data {
        // Write ID cols
        for (i, val) in key.iter().enumerate() {
            if i > 0 {
                write!(writer, "\t")?;
            }
            write!(writer, "{}", val)?;
        }

        // Write Value cols
        for name in &final_names {
            if !key.is_empty() {
                write!(writer, "\t")?;
            }
            if let Some(val) = row_map.get(name) {
                write!(writer, "{}", val)?;
            } else {
                write!(writer, "{}", config.fill_value)?;
            }
        }
        writeln!(writer)?;
    }

    Ok(())
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
