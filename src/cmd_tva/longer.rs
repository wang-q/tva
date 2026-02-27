use crate::libs::tsv::fields;
use crate::libs::tsv::fields::Header;
use clap::*;
use regex::Regex;
use std::collections::HashSet;
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("longer")
        .about("Reshapes a wide table into a long format")
        .after_help(include_str!("../../docs/help/longer.md"))
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
                .num_args(1..)
                .default_value("name")
                .help("Name of the new key column(s)"),
        )
        .arg(
            Arg::new("values-to")
                .long("values-to")
                .default_value("value")
                .help("Name of the new value column"),
        )
        .arg(
            Arg::new("names-pattern")
                .long("names-pattern")
                .help("Regex with capture groups to extract parts of column names"),
        )
        .arg(
            Arg::new("names-sep")
                .long("names-sep")
                .help("Separator to split column names into multiple columns"),
        )
        .arg(
            Arg::new("values-drop-na")
                .long("values-drop-na")
                .action(ArgAction::SetTrue)
                .help("Drop rows where the value is empty"),
        )
        .arg(
            Arg::new("names-prefix")
                .long("names-prefix")
                .help("A string to remove from the start of each variable name"),
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
    let names_to: Vec<&String> = args.get_many::<String>("names-to").unwrap().collect();
    let values_to = args.get_one::<String>("values-to").unwrap();
    let drop_na = args.get_flag("values-drop-na");
    let names_prefix = args.get_one::<String>("names-prefix");
    let names_sep = args.get_one::<String>("names-sep");
    let names_pattern = args
        .get_one::<String>("names-pattern")
        .map(|s| Regex::new(s))
        .transpose()?;

    if names_to.len() > 1 && names_sep.is_none() && names_pattern.is_none() {
        return Err(anyhow::anyhow!(
            "Multiple names-to provided but neither --names-sep nor --names-pattern is specified"
        ));
    }

    let mut melt_indices: Option<Vec<usize>> = None;
    let mut id_indices: Option<Vec<usize>> = None;
    // Pre-computed bytes for the "name" column(s) for each melt index
    // Each entry corresponds to a melt_index and contains the tab-joined name columns as bytes
    let mut melt_name_bytes: Option<Vec<Vec<u8>>> = None;

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut reader = crate::libs::tsv::reader::TsvReader::new(input.reader);

        // Read header
        let header_bytes_opt = reader.read_header()?;
        if header_bytes_opt.is_none() {
            continue;
        }
        let header_bytes = header_bytes_opt.unwrap();
        let current_header_fields: Vec<String> = String::from_utf8_lossy(&header_bytes)
            .trim_end_matches(&['\r', '\n'][..])
            .split('\t')
            .map(|s| s.to_string())
            .collect();

        // Initialize indices from the first file
        if melt_indices.is_none() {
            let header = Header::from_fields(current_header_fields.clone());

            let melt_indices_1based =
                fields::parse_field_list_with_header(cols_spec, Some(&header), '\t')
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

            let m_indices: Vec<usize> =
                melt_indices_1based.iter().map(|&i| i - 1).collect();
            let melt_set: HashSet<usize> = m_indices.iter().cloned().collect();

            // Identify id columns (those not in melt_indices)
            let mut i_indices: Vec<usize> = Vec::new();
            for i in 0..current_header_fields.len() {
                if !melt_set.contains(&i) {
                    i_indices.push(i);
                }
            }

            // Write new header
            let mut new_header_fields: Vec<String> =
                Vec::with_capacity(i_indices.len() + names_to.len() + 1);
            for &i in &i_indices {
                new_header_fields.push(current_header_fields[i].clone());
            }
            for name in &names_to {
                new_header_fields.push(name.to_string());
            }
            new_header_fields.push(values_to.clone());
            writeln!(writer, "{}", new_header_fields.join("\t"))?;

            // Pre-calculate name column bytes for each melt index
            let mut name_bytes_vec = Vec::with_capacity(m_indices.len());
            for &idx in &m_indices {
                let mut name_part = current_header_fields[idx].as_str();
                if let Some(prefix) = names_prefix {
                    if let Some(stripped) = name_part.strip_prefix(prefix) {
                        name_part = stripped;
                    }
                }

                let mut computed_name = String::new();
                if let Some(regex) = &names_pattern {
                     if let Some(caps) = regex.captures(name_part) {
                        for i in 1..=names_to.len() {
                            if i > 1 {
                                computed_name.push('\t');
                            }
                            if let Some(m) = caps.get(i) {
                                computed_name.push_str(m.as_str());
                            }
                        }
                    } else {
                        // Fallback: write original name in first col, tabs for others
                        computed_name.push_str(name_part);
                        for _ in 1..names_to.len() {
                             computed_name.push('\t');
                        }
                    }
                } else if let Some(sep) = names_sep {
                    let parts: Vec<&str> = name_part.split(sep).collect();
                    for i in 0..names_to.len() {
                        if i > 0 {
                            computed_name.push('\t');
                        }
                        if i < parts.len() {
                            computed_name.push_str(parts[i]);
                        }
                    }
                } else {
                    computed_name.push_str(name_part);
                }
                name_bytes_vec.push(computed_name.into_bytes());
            }

            melt_indices = Some(m_indices);
            id_indices = Some(i_indices);
            melt_name_bytes = Some(name_bytes_vec);
        }

        let m_indices = melt_indices.as_ref().unwrap();
        let i_indices = id_indices.as_ref().unwrap();
        let name_bytes = melt_name_bytes.as_ref().unwrap();
        let mut field_buf: Vec<usize> = Vec::with_capacity(current_header_fields.len());
        
        // Process remaining rows
        reader.for_each_record(|line| {
            if line.is_empty() {
                return Ok(());
            }

            // Split line into fields (store indices in buffer)
            field_buf.clear();
            for pos in memchr::memchr_iter(b'\t', line) {
                field_buf.push(pos);
            }
            let ends = &field_buf;
            let len = ends.len() + 1;

            // Helper to get field bytes
            let get_field = |idx: usize| -> &[u8] {
                if idx >= len {
                    return b"";
                }
                let end = if idx < ends.len() { ends[idx] } else { line.len() };
                let start = if idx == 0 { 0 } else { ends[idx - 1] + 1 };
                &line[start..end]
            };

            // Iterate over melt columns
            for (k, &melt_idx) in m_indices.iter().enumerate() {
                let value = get_field(melt_idx);
                
                if drop_na && value.is_empty() {
                    continue;
                }

                // Write ID columns
                for (j, &id_idx) in i_indices.iter().enumerate() {
                    if j > 0 {
                        writer.write_all(b"\t")?;
                    }
                    writer.write_all(get_field(id_idx))?;
                }

                // If we wrote ID columns, add tab separator
                if !i_indices.is_empty() {
                    writer.write_all(b"\t")?;
                }

                // Write name column(s) (pre-calculated)
                writer.write_all(&name_bytes[k])?;

                // Write value
                writer.write_all(b"\t")?;
                writer.write_all(value)?;
                writer.write_all(b"\n")?;
            }

            Ok(())
        })?;
    }

    Ok(())
}

// Struct ProcessRowConfig and fn process_row are removed as they are inlined

