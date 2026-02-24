use clap::*;
use crate::libs::fields;
use crate::libs::fields::Header;
use std::collections::HashSet;
use std::io::{BufRead, Write};

pub fn make_subcommand() -> Command {
    Command::new("longer")
        .about("Reshapes a wide table into a long format")
        .after_help(
            r###"
Reshapes a table from wide to long format by gathering multiple columns into
key-value pairs. This command is useful for "tidying" data where some column
names are actually values of a variable.

Input:
- Reads from one or more TSV files or standard input.
- Files ending in '.gz' are transparently decompressed.
- When multiple files are provided, the first file's header determines the
  schema (columns to reshape). Subsequent files must have the same column
  structure; their headers are skipped.

Reshaping behavior:
- --cols / -c
  Specifies which columns to reshape (melt). Can be column names, indices
  (1-based), or ranges (e.g., '3-5').
- --names-to
  The name of the new column that will contain the original column headers.
- --values-to
  The name of the new column that will contain the data values.
- --values-drop-na
  If set, rows where the value is empty will be omitted from the output.

Examples:
1. Reshape columns 3, 4, and 5 into default "name" and "value" columns
   tva longer data.tsv --cols 3-5

2. Reshape columns starting with "wk", specifying new column names
   tva longer data.tsv --cols "wk*" --names-to week --values-to rank

3. Reshape all columns except the first two
   tva longer data.tsv --cols 3-

4. Process multiple files and save to output
   tva longer data1.tsv data2.tsv --cols 2-5 --outfile result.tsv
"###,
        )
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("cols")
                .long("cols")
                .short('c')
                .required(true)
                .help("Columns to reshape"),
        )
        .arg(
            Arg::new("names-to")
                .long("names-to")
                .default_value("name")
                .help("Name of the new key column"),
        )
        .arg(
            Arg::new("values-to")
                .long("values-to")
                .default_value("value")
                .help("Name of the new value column"),
        )
        .arg(
            Arg::new("values-drop-na")
                .long("values-drop-na")
                .action(ArgAction::SetTrue)
                .help("Drop rows where the value is empty"),
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

    let cols_spec = args.get_one::<String>("cols").unwrap();
    let names_to = args.get_one::<String>("names-to").unwrap();
    let values_to = args.get_one::<String>("values-to").unwrap();
    let drop_na = args.get_flag("values-drop-na");

    let mut melt_indices: Option<Vec<usize>> = None;
    let mut id_indices: Option<Vec<usize>> = None;
    let mut header_fields: Option<Vec<String>> = None;

    for infile in &infiles {
        let is_stdin = infile == "stdin";
        if !is_stdin {
            if !crate::libs::io::has_nonempty_line(infile)? {
                continue;
            }
        }

        let mut reader = crate::libs::io::reader(infile);

        // Read header
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            continue;
        }
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        let current_header_fields: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();

        // Initialize indices from the first file
        if melt_indices.is_none() {
            let header = Header::from_fields(current_header_fields.clone());

            // Parse columns to melt
            let melt_indices_1based = fields::parse_field_list_with_header(cols_spec, Some(&header), '\t')
                .map_err(|e| anyhow::anyhow!(e))?;

            // Check if any index is out of bounds
            let max_idx = current_header_fields.len();
            for &idx in &melt_indices_1based {
                if idx > max_idx {
                    return Err(anyhow::anyhow!(
                        "Invalid column index {}: input file has only {} columns",
                        idx,
                        max_idx
                    ));
                }
            }

            let m_indices: Vec<usize> = melt_indices_1based.iter().map(|&i| i - 1).collect();
            let melt_set: HashSet<usize> = m_indices.iter().cloned().collect();

            // Identify id columns (those not in melt_indices)
            let mut i_indices: Vec<usize> = Vec::new();
            for i in 0..current_header_fields.len() {
                if !melt_set.contains(&i) {
                    i_indices.push(i);
                }
            }

            // Write new header
            let mut new_header_fields: Vec<String> = Vec::with_capacity(i_indices.len() + 2);
            for &i in &i_indices {
                new_header_fields.push(current_header_fields[i].clone());
            }
            new_header_fields.push(names_to.clone());
            new_header_fields.push(values_to.clone());
            writeln!(writer, "{}", new_header_fields.join("\t"))?;

            melt_indices = Some(m_indices);
            id_indices = Some(i_indices);
            header_fields = Some(current_header_fields);
        } else {
            // For subsequent files, we skip the header line (which we just read into `line`)
            // We assume the schema matches the first file
        }

        let m_indices = melt_indices.as_ref().unwrap();
        let i_indices = id_indices.as_ref().unwrap();
        let h_fields = header_fields.as_ref().unwrap();

        // Process rows
        for line in reader.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split('\t').collect();

            // Pre-build id part of the output line
            let mut id_parts: Vec<&str> = Vec::with_capacity(i_indices.len());
            for &i in i_indices {
                if i < fields.len() {
                    id_parts.push(fields[i]);
                } else {
                    id_parts.push("");
                }
            }

            for &melt_idx in m_indices {
                let value = if melt_idx < fields.len() {
                    fields[melt_idx]
                } else {
                    ""
                };

                if drop_na && value.is_empty() {
                    continue;
                }

                // Write output row
                for (j, part) in id_parts.iter().enumerate() {
                    if j > 0 {
                        write!(writer, "\t")?;
                    }
                    write!(writer, "{}", part)?;
                }
                // If there were no id columns, we don't need a leading tab
                if !id_parts.is_empty() {
                    write!(writer, "\t")?;
                }

                // Write variable name and value
                // Use the header fields from the FIRST file for consistency of variable names
                write!(writer, "{}\t{}", h_fields[melt_idx], value)?;
                writeln!(writer)?;
            }
        }
    }

    Ok(())
}
