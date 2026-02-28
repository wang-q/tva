use crate::libs::io::map_io_err;
use crate::libs::stats::{Aggregator, OpKind, Operation, StatsProcessor};
use crate::libs::tsv::fields;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::key::{KeyExtractor, KeyBuffer};
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::HashMap;

pub fn make_subcommand() -> Command {
    Command::new("stats")
        .about("Calculates summary statistics (like tsv-summarize)")
        .after_help(include_str!("../../docs/help/stats.md"))
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .action(ArgAction::SetTrue)
                .help("Treat the first line of each file as a header"),
        )
        .arg(
            Arg::new("group-by")
                .long("group-by")
                .short('g')
                .num_args(1)
                .help("Fields to group by"),
        )
        .arg(
            Arg::new("count")
                .long("count")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Count the number of rows"),
        )
        .arg(
            Arg::new("sum")
                .long("sum")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate sum of fields"),
        )
        .arg(
            Arg::new("mean")
                .long("mean")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate mean of fields"),
        )
        .arg(
            Arg::new("min")
                .long("min")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate min of fields"),
        )
        .arg(
            Arg::new("max")
                .long("max")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate max of fields"),
        )
        .arg(
            Arg::new("median")
                .long("median")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate median of fields"),
        )
        .arg(
            Arg::new("stdev")
                .long("stdev")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate standard deviation of fields"),
        )
        .arg(
            Arg::new("variance")
                .long("variance")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate variance of fields"),
        )
        .arg(
            Arg::new("mad")
                .long("mad")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate median absolute deviation of fields"),
        )
        .arg(
            Arg::new("first")
                .long("first")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Get the first value of fields"),
        )
        .arg(
            Arg::new("last")
                .long("last")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Get the last value of fields"),
        )
        .arg(
            Arg::new("nunique")
                .long("nunique")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Count the number of unique values"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Get the most frequent value (mode)"),
        )
        .arg(
            Arg::new("geomean")
                .long("geomean")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate geometric mean of fields"),
        )
        .arg(
            Arg::new("harmmean")
                .long("harmmean")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate harmonic mean of fields"),
        )
        .arg(
            Arg::new("q1")
                .long("q1")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate 1st quartile (25th percentile) of fields"),
        )
        .arg(
            Arg::new("q3")
                .long("q3")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate 3rd quartile (75th percentile) of fields"),
        )
        .arg(
            Arg::new("iqr")
                .long("iqr")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate interquartile range (Q3-Q1) of fields"),
        )
        .arg(
            Arg::new("cv")
                .long("cv")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate coefficient of variation (stdev/mean) of fields"),
        )
        .arg(
            Arg::new("range")
                .long("range")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate range (max-min) of fields"),
        )
        .arg(
            Arg::new("unique")
                .long("unique")
                .num_args(1)
                .action(ArgAction::Append)
                .help("List unique values of fields (comma separated)"),
        )
        .arg(
            Arg::new("collapse")
                .long("collapse")
                .num_args(1)
                .action(ArgAction::Append)
                .help("List all values of fields (comma separated)"),
        )
        .arg(
            Arg::new("rand")
                .long("rand")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Pick a random value from fields"),
        )
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
}

struct OpConfig {
    kind: OpKind,
    spec: Option<String>,
    arg_index: usize,
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    let mut op_configs = Vec::new();

    // Collect operations
    if let Some(indices) = matches.indices_of("sum") {
        for (i, val) in indices.zip(matches.get_many::<String>("sum").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Sum,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("mean") {
        for (i, val) in indices.zip(matches.get_many::<String>("mean").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Mean,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("min") {
        for (i, val) in indices.zip(matches.get_many::<String>("min").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Min,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("max") {
        for (i, val) in indices.zip(matches.get_many::<String>("max").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Max,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("median") {
        for (i, val) in indices.zip(matches.get_many::<String>("median").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Median,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("stdev") {
        for (i, val) in indices.zip(matches.get_many::<String>("stdev").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Stdev,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("variance") {
        for (i, val) in indices.zip(matches.get_many::<String>("variance").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Variance,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("mad") {
        for (i, val) in indices.zip(matches.get_many::<String>("mad").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Mad,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("first") {
        for (i, val) in indices.zip(matches.get_many::<String>("first").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::First,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("last") {
        for (i, val) in indices.zip(matches.get_many::<String>("last").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Last,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("nunique") {
        for (i, val) in indices.zip(matches.get_many::<String>("nunique").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::NUnique,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("mode") {
        for (i, val) in indices.zip(matches.get_many::<String>("mode").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Mode,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("geomean") {
        for (i, val) in indices.zip(matches.get_many::<String>("geomean").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::GeoMean,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("harmmean") {
        for (i, val) in indices.zip(matches.get_many::<String>("harmmean").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::HarmMean,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("q1") {
        for (i, val) in indices.zip(matches.get_many::<String>("q1").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Q1,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("q3") {
        for (i, val) in indices.zip(matches.get_many::<String>("q3").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Q3,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("iqr") {
        for (i, val) in indices.zip(matches.get_many::<String>("iqr").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::IQR,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("cv") {
        for (i, val) in indices.zip(matches.get_many::<String>("cv").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::CV,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("range") {
        for (i, val) in indices.zip(matches.get_many::<String>("range").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Range,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("unique") {
        for (i, val) in indices.zip(matches.get_many::<String>("unique").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Unique,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("collapse") {
        for (i, val) in indices.zip(matches.get_many::<String>("collapse").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Collapse,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("rand") {
        for (i, val) in indices.zip(matches.get_many::<String>("rand").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Rand,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }

    // Handle count.
    if matches.get_flag("count") {
        op_configs.push(OpConfig {
            kind: OpKind::Count,
            spec: None,
            arg_index: 0,
        });
    }

    op_configs.sort_by_key(|c| c.arg_index);

    let infiles: Vec<String> = match matches.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let header_mode = matches.get_flag("header");
    let group_by_spec = matches.get_one::<String>("group-by").cloned();

    let mut processor: Option<StatsProcessor> = None;
    let mut aggregator: Option<Aggregator> = None;
    let mut groups: HashMap<KeyBuffer, Aggregator> = HashMap::new();
    let mut group_extractor: Option<KeyExtractor> = None;
    let mut use_grouping = false;

    // Helper to setup processor
    let setup_processor = |header_opt: Option<&fields::Header>| -> anyhow::Result<(StatsProcessor, Option<KeyExtractor>, Vec<String>)> {
        let mut ops = Vec::new();
        let mut output_headers = Vec::new();

        for config in &op_configs {
            if let OpKind::Count = config.kind {
                ops.push(Operation {
                    kind: OpKind::Count,
                    field_idx: None,
                });
                output_headers.push("count".to_string());
            } else if let Some(spec) = &config.spec {
                let indices = fields::parse_field_list_with_header(
                    spec,
                    header_opt,
                    '\t',
                )
                .map_err(|e| anyhow::anyhow!("Error parsing field list: {}", e))?;

                for idx in indices {
                    let field_idx = idx - 1;
                    ops.push(Operation {
                        kind: config.kind.clone(),
                        field_idx: Some(field_idx),
                    });

                    let suffix = match config.kind {
                        OpKind::Sum => "_sum",
                        OpKind::Mean => "_mean",
                        OpKind::Min => "_min",
                        OpKind::Max => "_max",
                        OpKind::Median => "_median",
                        OpKind::Stdev => "_stdev",
                        OpKind::Variance => "_variance",
                        OpKind::Mad => "_mad",
                        OpKind::First => "_first",
                        OpKind::Last => "_last",
                        OpKind::NUnique => "_nunique",
                        OpKind::Mode => "_mode",
                        OpKind::GeoMean => "_geomean",
                        OpKind::HarmMean => "_harmmean",
                        OpKind::Q1 => "_q1",
                        OpKind::Q3 => "_q3",
                        OpKind::IQR => "_iqr",
                        OpKind::CV => "_cv",
                        OpKind::Range => "_range",
                        OpKind::Unique => "_unique",
                        OpKind::Collapse => "_collapse",
                        OpKind::Rand => "_rand",
                        OpKind::Count => "",
                    };

                    let name = if let Some(h) = header_opt {
                        if field_idx < h.fields.len() {
                            format!("{}{}", h.fields[field_idx], suffix)
                        } else {
                            format!("field{}{}", idx, suffix)
                        }
                    } else {
                        format!("field{}{}", idx, suffix)
                    };
                    output_headers.push(name);
                }
            }
        }

        let extractor = if let Some(spec) = &group_by_spec {
            let idxs = fields::parse_field_list_with_header(spec, header_opt, '\t')
                .map_err(|e| anyhow::anyhow!("Error parsing group-by fields: {}", e))?;
                // .into_iter()
                // .map(|i| i - 1) // KeyExtractor now takes 1-based indices!
                // .collect::<Vec<_>>();
            if idxs.is_empty() {
                None
            } else {
                Some(KeyExtractor::new(Some(idxs), false, false)) // strict=false for stats
            }
        } else {
            None
        };

        let mut final_headers = Vec::new();
        if let Some(extractor) = &extractor {
            if let Some(indices) = &extractor.indices {
                for &idx in indices {
                    // indices are 1-based
                    if let Some(h) = header_opt {
                        if idx > 0 && idx <= h.fields.len() {
                            final_headers.push(h.fields[idx - 1].to_string());
                        } else {
                            final_headers.push(format!("field{}", idx));
                        }
                    } else {
                        final_headers.push(format!("field{}", idx));
                    }
                }
            }
        }
        final_headers.extend(output_headers);

        Ok((StatsProcessor::new(ops), extractor, final_headers))
    };

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut reader = TsvReader::with_capacity(input.reader, 512 * 1024);

        if header_mode {
            let header_bytes_opt = reader.read_header().map_err(map_io_err)?;
            if processor.is_none() {
                if let Some(header_bytes) = header_bytes_opt {
                    let line = String::from_utf8_lossy(&header_bytes);
                    let header = fields::Header::from_line(&line, '\t');
                    
                    let (proc, extractor, headers) = setup_processor(Some(&header))?;
                    processor = Some(proc);
                    group_extractor = extractor;
                    use_grouping = group_extractor.is_some();
                    
                    if !use_grouping {
                        aggregator = Some(processor.as_ref().unwrap().create_aggregator());
                    }
                    
                    println!("{}", headers.join("\t"));
                } else {
                    // Empty file with --header.
                    // Attempt to setup processor without header info (will fail if named fields are used).
                    let (proc, extractor, headers) = setup_processor(None)?;
                    processor = Some(proc);
                    group_extractor = extractor;
                    use_grouping = group_extractor.is_some();
                    
                    if !use_grouping {
                        aggregator = Some(processor.as_ref().unwrap().create_aggregator());
                    }
                    
                    println!("{}", headers.join("\t"));
                }
            }
        } else {
            if processor.is_none() {
                let (proc, extractor, _) = setup_processor(None)?;
                processor = Some(proc);
                group_extractor = extractor;
                use_grouping = group_extractor.is_some();
                
                if !use_grouping {
                    aggregator = Some(processor.as_ref().unwrap().create_aggregator());
                }
            }
        }
        
        if let Some(proc) = &processor {
            reader.for_each_row(|row| {
                if use_grouping {
                    let key_res = group_extractor.as_mut().unwrap().extract_from_row(row, b'\t');
                    let key = match key_res {
                        Ok(k) => k.into_owned(),
                        Err(_) => KeyBuffer::new(),
                    };
                    
                    let agg = groups.entry(key).or_insert_with(|| proc.create_aggregator());
                    proc.update(agg, row);
                } else {
                    if let Some(agg) = &mut aggregator {
                        proc.update(agg, row);
                    }
                }
                Ok(())
            }).map_err(map_io_err)?;
        }
    }

    if let Some(proc) = &processor {
        if use_grouping {
            let mut keys: Vec<_> = groups.keys().collect();
            keys.sort();

            for key in keys {
                let agg = &groups[key];
                print!("{}", String::from_utf8_lossy(key));

                let values = proc.format_results(agg);
                if !values.is_empty() {
                    print!("\t{}", values.join("\t"));
                }
                println!();
            }
        } else {
            if let Some(agg) = &aggregator {
                let values = proc.format_results(agg);
                println!("{}", values.join("\t"));
            }
        }
    }

    Ok(())
}
