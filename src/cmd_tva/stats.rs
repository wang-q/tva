use crate::libs::io::reader;
use crate::libs::stats::{Aggregator, OpKind, Operation, StatsProcessor};
use crate::libs::tsv::fields;
use crate::libs::tsv::reader::TsvReader;
use crate::libs::tsv::record::Row;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::HashMap;
use std::io::BufRead;

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

    let mut reader = reader(&infiles[0]);
    let mut header: Option<fields::Header> = None;

    if header_mode {
        let mut line = String::new();
        if reader.read_line(&mut line)? > 0 {
            header = Some(fields::Header::from_line(line.trim_end(), '\t'));
        }
    }

    // Resolve operations
    let mut ops = Vec::new();
    let mut output_headers = Vec::new();

    for config in &op_configs {
        match config.kind {
            OpKind::Count => {
                ops.push(Operation {
                    kind: OpKind::Count,
                    field_idx: None,
                });
                output_headers.push("count".to_string());
            }
            _ => {
                if let Some(spec) = &config.spec {
                    let indices = fields::parse_field_list_with_header(
                        spec,
                        header.as_ref(),
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
                            _ => "",
                        };

                        let name = if let Some(h) = &header {
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
        }
    }

    let group_indices = if let Some(spec) = &group_by_spec {
        fields::parse_field_list_with_header(spec, header.as_ref(), '\t')
            .map_err(|e| anyhow::anyhow!("Error parsing group-by fields: {}", e))?
            .into_iter()
            .map(|i| i - 1)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if header_mode {
        let mut final_headers = Vec::new();
        if let Some(h) = &header {
            for &idx in &group_indices {
                if idx < h.fields.len() {
                    final_headers.push(h.fields[idx].clone());
                } else {
                    final_headers.push(format!("field{}", idx + 1));
                }
            }
        }
        final_headers.extend(output_headers);
        println!("{}", final_headers.join("\t"));
    }

    let use_grouping = !group_indices.is_empty();
    let mut first_reader = Some(reader);

    let processor = StatsProcessor::new(ops);

    if use_grouping {
        let mut groups: HashMap<Vec<u8>, Aggregator> = HashMap::new();

        for (i, infile) in infiles.iter().enumerate() {
            let mut file_reader = if i == 0 {
                first_reader.take().unwrap()
            } else {
                let mut r = crate::libs::io::reader(infile);
                if header_mode {
                    let mut dummy = String::new();
                    r.read_line(&mut dummy)?;
                }
                r
            };

            let mut tsv_reader = TsvReader::new(file_reader);
            tsv_reader.for_each_row(|row| {
                let mut key = Vec::new();
                for (k_i, &idx) in group_indices.iter().enumerate() {
                    if k_i > 0 {
                        key.push(b'\t');
                    }
                    if let Some(field) = row.get_bytes(idx + 1) {
                        key.extend_from_slice(field);
                    }
                }

                let aggregator = groups
                    .entry(key)
                    .or_insert_with(|| processor.create_aggregator());
                processor.update(aggregator, row);

                Ok(())
            })?;
        }

        let mut keys: Vec<_> = groups.keys().collect();
        keys.sort();

        for key in keys {
            let agg = &groups[key];
            print!("{}", String::from_utf8_lossy(key));

            let values = processor.format_results(agg);
            if !values.is_empty() {
                print!("\t{}", values.join("\t"));
            }
            println!();
        }
    } else {
        let mut aggregator = processor.create_aggregator();

        for (i, infile) in infiles.iter().enumerate() {
            let mut file_reader = if i == 0 {
                first_reader.take().unwrap()
            } else {
                let mut r = crate::libs::io::reader(infile);
                if header_mode {
                    let mut dummy = String::new();
                    r.read_line(&mut dummy)?;
                }
                r
            };

            let mut tsv_reader = TsvReader::new(file_reader);
            tsv_reader.for_each_row(|row| {
                processor.update(&mut aggregator, row);
                Ok(())
            })?;
        }

        if !processor.ops.is_empty() {
            let values = processor.format_results(&aggregator);
            println!("{}", values.join("\t"));
        }
    }

    Ok(())
}
