use clap::*;
use std::collections::HashSet;
use std::ops::Range;

pub fn make_subcommand() -> Command {
    Command::new("select")
        .about("Selects and reorders TSV fields")
        .after_help(include_str!("../../docs/help/select.md"))
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
            Arg::new("rest")
                .long("rest")
                .short('r')
                .value_parser(["first", "last", "none"])
                .num_args(1)
                .help("Output location for fields not included in --fields"),
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

use crate::libs::io::map_io_err;
use crate::libs::tsv::select;
use crate::libs::tsv::select::SelectPlan;

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

    let rest_arg = args.get_one::<String>("rest").map(|s| s.as_str());
    let mut rest_mode = match rest_arg {
        Some("first") => select::RestMode::First,
        Some("last") => select::RestMode::Last,
        Some("none") => select::RestMode::None,
        _ => select::RestMode::None,
    };

    if fields_spec.is_some() && exclude_spec.is_some() {
        if rest_arg.is_none() {
            rest_mode = select::RestMode::Last;
        }
    } else if fields_spec.is_none() && exclude_spec.is_some() {
        if rest_arg.is_none() {
            rest_mode = select::RestMode::First;
        }
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

    let mut select_plan: Option<SelectPlan> = None;
    let mut output_ranges: Vec<Range<usize>> = Vec::new();

    let delim_byte = delimiter as u8;

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut tsv_reader =
            crate::libs::tsv::reader::TsvReader::with_capacity(input.reader, 512 * 1024);

        if has_header {
            if let Some(header_bytes) = tsv_reader.read_header().map_err(map_io_err)? {
                if !header_written {
                    let line_str = String::from_utf8_lossy(&header_bytes);
                    let header = crate::libs::tsv::fields::Header::from_line(
                        &line_str, delimiter,
                    );

                    if let Some(ref spec) = fields_spec {
                        field_indices = Some(
                            crate::libs::tsv::fields::parse_field_list_with_header_preserve_order(
                                spec,
                                Some(&header),
                                delimiter,
                            )
                            .map_err(map_io_err)?,
                        );
                    }
                    if let Some(ref spec) = exclude_spec {
                        let indices =
                            crate::libs::tsv::fields::parse_field_list_with_header_preserve_order(
                                spec,
                                Some(&header),
                                delimiter,
                            )
                            .map_err(map_io_err)?;
                        exclude_indices = Some(indices.clone());
                        exclude_set = Some(indices.into_iter().collect());
                    }

                    if let (Some(ref f), Some(ref e)) = (&field_indices, &exclude_set) {
                        for idx in f {
                            if e.contains(idx) {
                                arg_error(&format!(
                                    "Field {} is both selected and excluded",
                                    idx
                                ));
                            }
                        }
                    }

                    let empty_vec = Vec::new();
                    let f_indices = field_indices.as_ref().unwrap_or(&empty_vec);

                    select::write_with_rest(
                        &mut writer,
                        &header_bytes,
                        delim_byte,
                        f_indices,
                        exclude_set.as_ref(),
                        rest_mode,
                    )
                    .map_err(map_io_err)?;

                    header_written = true;
                }
            } else {
                continue;
            }
        }

        if fields_spec.is_some() && field_indices.is_none() {
            if let Some(ref spec) = fields_spec {
                field_indices = Some(
                    crate::libs::tsv::fields::parse_field_list_with_header_preserve_order(
                        spec, None, delimiter,
                    )
                    .map_err(map_io_err)?,
                );
            }
        }

        if exclude_spec.is_some() && exclude_set.is_none() {
            if let Some(ref indices) = exclude_indices {
                exclude_set = Some(indices.iter().copied().collect());
            } else if let Some(ref spec) = exclude_spec {
                let indices =
                    crate::libs::tsv::fields::parse_field_list_with_header_preserve_order(
                        spec, None, delimiter,
                    )
                    .map_err(map_io_err)?;
                exclude_indices = Some(indices.clone());
                exclude_set = Some(indices.into_iter().collect());
            }
        }

        if let (Some(ref f), Some(ref e)) = (&field_indices, &exclude_set) {
            for idx in f {
                if e.contains(idx) {
                    arg_error(&format!("Field {} is both selected and excluded", idx));
                }
            }
        }

        tsv_reader.for_each_record(|line_bytes| {
            if line_bytes.is_empty() {
                return Ok(());
            }

            if rest_mode == select::RestMode::None && exclude_set.is_none() {
                if let Some(ref idxs) = field_indices {
                    if select_plan.is_none() {
                        select_plan = Some(SelectPlan::new(idxs));
                    }
                    select::write_selected_from_bytes(
                        &mut writer,
                        line_bytes,
                        delim_byte,
                        select_plan.as_ref().unwrap(),
                        &mut output_ranges,
                    )?;
                } else {
                    // Should not happen if one of -f or -e required
                    // If -f not present, and -e not present, we error early.
                    // Here we are inside loop.
                    // If -e not present, and rest_mode None, implies -f present.
                }
            } else {
                // Use generic write_with_rest
                let empty_vec = Vec::new();
                let f_indices = field_indices.as_ref().unwrap_or(&empty_vec);

                select::write_with_rest(
                    &mut writer,
                    line_bytes,
                    delim_byte,
                    f_indices,
                    exclude_set.as_ref(),
                    rest_mode,
                )?;
            }
            Ok(())
        })?;
    }

    Ok(())
}
