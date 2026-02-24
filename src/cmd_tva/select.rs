use clap::*;
use std::collections::HashSet;
use std::io::BufRead;

pub fn make_subcommand() -> Command {
    Command::new("select")
        .about("Selects and reorders TSV fields")
        .after_help(
            r###"
Reads TSV data from files or standard input and writes selected fields to
standard output.

Fields can be specified by number or, when a header line is present, by
field name. Field numbers are 1-based and support ranges, for example
1,3-5. When --header is set, field names from the first header line can
be used in the field list.

Input:
- If no input files are given, or an input file is 'stdin', data is read
  from standard input.
- Files ending in '.gz' are transparently decompressed.

Selection:
- One of --fields/-f or --exclude/-e is required.
- --fields/-f keeps only the listed fields, in the order given.
- --exclude/-e drops the listed fields and keeps all others.
- In header mode, field names and numeric indices can be mixed.

Field syntax:
- Field lists support 1-based indices, ranges (1-3,5-7), header names,
  name ranges (run-user_time), and wildcards (*_time).
- Run `tva --help-fields` for a full description shared across tva commands.

Output:
- By default, output is written to standard output.
- Use --outfile to write to a file instead.
"###,
        )
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
        .arg(
            Arg::new("fields")
                .long("fields")
                .short('f')
                .num_args(1)
                .help("Field list to keep, using 1-based indices or names"),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .num_args(1)
                .help("Field list to drop from the output"),
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
                .short('d')
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

fn arg_error(msg: &str) -> ! {
    eprintln!("tva select: {}", msg);
    std::process::exit(1);
}

pub fn execute(args: &ArgMatches) -> anyhow::Result<()> {
    let mut writer = crate::libs::io::writer(args.get_one::<String>("outfile").unwrap());

    let infiles: Vec<String> = match args.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let fields_spec: Option<String> = args.get_one::<String>("fields").cloned();
    let exclude_spec: Option<String> = args.get_one::<String>("exclude").cloned();

    if fields_spec.is_none() && exclude_spec.is_none() {
        arg_error("one of --fields/-f or --exclude/-e is required");
    }

    if fields_spec.is_some() && exclude_spec.is_some() {
        arg_error("--fields/-f and --exclude/-e cannot be used together");
    }

    let has_header = args.get_flag("header");

    let delimiter_str = args
        .get_one::<String>("delimiter")
        .cloned()
        .unwrap_or_else(|| "\t".to_string());
    let mut chars = delimiter_str.chars();
    let delimiter = chars.next().unwrap_or('\t');
    if chars.next().is_some() {
        arg_error(&format!(
            "delimiter must be a single character, got `{}`",
            delimiter_str
        ));
    }

    let mut header_written = false;
    let mut field_indices: Option<Vec<usize>> = None;
    let mut exclude_indices: Option<Vec<usize>> = None;
    let mut exclude_set: Option<HashSet<usize>> = None;

    for input in crate::libs::io::input_sources(&infiles) {
        let reader = input.reader;
        let mut is_first_line = true;

        for line in reader.lines().map_while(Result::ok) {
            let mut line = line;
            if let Some('\r') = line.chars().last() {
                line.pop();
            }

            if has_header && is_first_line {
                if !header_written {
                    let header =
                        crate::libs::fields::Header::from_line(&line, delimiter);

                    if let Some(ref spec) = fields_spec {
                        field_indices = Some(
                            crate::libs::fields::parse_field_list_with_header_preserve_order(
                                spec,
                                Some(&header),
                                delimiter,
                            )
                            .map_err(|e| anyhow::anyhow!(e))?,
                        );
                    } else if let Some(ref spec) = exclude_spec {
                        let indices =
                            crate::libs::fields::parse_field_list_with_header_preserve_order(
                                spec,
                                Some(&header),
                                delimiter,
                            )
                            .map_err(|e| anyhow::anyhow!(e))?;
                        exclude_indices = Some(indices.clone());
                        exclude_set = Some(indices.into_iter().collect());
                    }

                    if let Some(ref idxs) = field_indices {
                        write_selected_fields(
                            &mut writer,
                            &header.fields,
                            idxs,
                            delimiter,
                        )?;
                    } else if let Some(ref ex_set) = exclude_set {
                        let total = header.fields.len();
                        let mut idxs: Vec<usize> = Vec::new();
                        for i in 1..=total {
                            if !ex_set.contains(&i) {
                                idxs.push(i);
                            }
                        }
                        write_selected_fields(
                            &mut writer,
                            &header.fields,
                            &idxs,
                            delimiter,
                        )?;
                        field_indices = Some(idxs);
                        exclude_set = None;
                    } else {
                        write_selected_fields(
                            &mut writer,
                            &header.fields,
                            &(1..=header.fields.len()).collect::<Vec<_>>(),
                            delimiter,
                        )?;
                    }

                    header_written = true;
                }

                is_first_line = false;
                continue;
            }

            is_first_line = false;

            if fields_spec.is_some() && field_indices.is_none() {
                if let Some(ref spec) = fields_spec {
                    field_indices = Some(
                        crate::libs::fields::parse_field_list_with_header_preserve_order(
                            spec,
                            None,
                            delimiter,
                        )
                        .map_err(|e| anyhow::anyhow!(e))?,
                    );
                }
            }

            if exclude_spec.is_some() && exclude_set.is_none() {
                if let Some(ref indices) = exclude_indices {
                    exclude_set = Some(indices.iter().copied().collect());
                } else if let Some(ref spec) = exclude_spec {
                    let indices = crate::libs::fields::parse_field_list_with_header_preserve_order(
                        spec,
                        None,
                        delimiter,
                    )
                    .map_err(|e| anyhow::anyhow!(e))?;
                    exclude_indices = Some(indices.clone());
                    exclude_set = Some(indices.into_iter().collect());
                }
            }

            if line.is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split(delimiter).collect();
            let field_count = fields.len();

            let selected_indices: Vec<usize> = if let Some(ref idxs) = field_indices {
                idxs.clone()
            } else if let Some(ref ex_set) = exclude_set {
                let mut v = Vec::new();
                for i in 1..=field_count {
                    if !ex_set.contains(&i) {
                        v.push(i);
                    }
                }
                v
            } else {
                (1..=field_count).collect()
            };

            if selected_indices.is_empty() {
                writer.write_all(b"\n")?;
                continue;
            }

            write_selected_fields_from_strs(
                &mut writer,
                &fields,
                &selected_indices,
                delimiter,
            )?;
        }
    }

    Ok(())
}

fn write_selected_fields_from_strs(
    writer: &mut dyn std::io::Write,
    fields: &[&str],
    indices: &[usize],
    delimiter: char,
) -> anyhow::Result<()> {
    let mut first = true;
    for idx in indices {
        if let Some(field) = fields.get(idx - 1) {
            if !first {
                writer.write_all(delimiter.to_string().as_bytes())?;
            }
            writer.write_all(field.as_bytes())?;
            first = false;
        }
    }
    writer.write_all(b"\n")?;
    Ok(())
}

fn write_selected_fields(
    writer: &mut dyn std::io::Write,
    fields: &[String],
    indices: &[usize],
    delimiter: char,
) -> anyhow::Result<()> {
    let mut first = true;
    for idx in indices {
        if let Some(field) = fields.get(idx - 1) {
            if !first {
                writer.write_all(delimiter.to_string().as_bytes())?;
            }
            writer.write_all(field.as_bytes())?;
            first = false;
        }
    }
    writer.write_all(b"\n")?;
    Ok(())
}
