use crate::libs::aggregation::{
    Aggregator, OpKind, Operation, StatsConfig, StatsProcessor,
};
use crate::libs::io::map_io_err;
use crate::libs::tsv::fields;
use crate::libs::tsv::key::{KeyBuffer, KeyExtractor};
use crate::libs::tsv::reader::TsvReader;
use clap::{Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexMap;

pub fn make_subcommand() -> Command {
    let mut cmd = Command::new("stats")
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
            Arg::new("delimiter")
                .long("delimiter")
                .short('d')
                .num_args(1)
                .default_value("\t")
                .help("Field delimiter"),
        )
        .arg(
            Arg::new("count")
                .long("count")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Count the number of rows"),
        );
    macro_rules! add_op_arg {
        ($cmd:ident, $name:expr, $help:expr) => {
            $cmd = $cmd.arg(
                Arg::new($name)
                    .long($name)
                    .num_args(1)
                    .action(ArgAction::Append)
                    .help($help),
            );
        };
        ($cmd:ident, $name:expr, $alias:expr, $help:expr) => {
            $cmd = $cmd.arg(
                Arg::new($name)
                    .long($name)
                    .visible_alias($alias)
                    .num_args(1)
                    .action(ArgAction::Append)
                    .help($help),
            );
        };
    }

    add_op_arg!(cmd, "sum", "Calculate sum of fields");
    add_op_arg!(cmd, "mean", "Calculate mean of fields");
    add_op_arg!(cmd, "min", "Calculate min of fields");
    add_op_arg!(cmd, "max", "Calculate max of fields");
    add_op_arg!(cmd, "median", "Calculate median of fields");
    add_op_arg!(cmd, "stdev", "Calculate standard deviation of fields");
    add_op_arg!(cmd, "variance", "var", "Calculate variance of fields");
    add_op_arg!(cmd, "mad", "Calculate median absolute deviation of fields");
    add_op_arg!(cmd, "first", "retain", "Get the first value of fields");
    add_op_arg!(cmd, "last", "Get the last value of fields");
    add_op_arg!(
        cmd,
        "nunique",
        "unique-count",
        "Count the number of unique values"
    );
    add_op_arg!(cmd, "mode", "Get the most frequent value (mode)");
    add_op_arg!(cmd, "geomean", "Calculate geometric mean of fields");
    add_op_arg!(cmd, "harmmean", "Calculate harmonic mean of fields");
    add_op_arg!(
        cmd,
        "q1",
        "Calculate 1st quartile (25th percentile) of fields"
    );
    add_op_arg!(
        cmd,
        "q3",
        "Calculate 3rd quartile (75th percentile) of fields"
    );
    add_op_arg!(
        cmd,
        "iqr",
        "Calculate interquartile range (Q3-Q1) of fields"
    );
    add_op_arg!(
        cmd,
        "cv",
        "Calculate coefficient of variation (stdev/mean) of fields"
    );
    add_op_arg!(cmd, "range", "Calculate range (max-min) of fields");
    add_op_arg!(
        cmd,
        "quantile",
        "Calculate quantiles (e.g. field:0.25,0.75)"
    );
    add_op_arg!(
        cmd,
        "values",
        "collapse",
        "List all values of fields (separated by --values-delimiter)"
    );
    add_op_arg!(
        cmd,
        "unique-values",
        "unique",
        "List unique values of fields (separated by --values-delimiter)"
    );
    add_op_arg!(cmd, "rand", "Pick a random value from fields");
    add_op_arg!(cmd, "mode-count", "Count of the most frequent value");
    add_op_arg!(cmd, "missing-count", "Number of missing (empty) fields");
    add_op_arg!(
        cmd,
        "not-missing-count",
        "Number of filled (non-empty) fields"
    );

    cmd = cmd
        .arg(
            Arg::new("write-header")
                .long("write-header")
                .short('w')
                .action(ArgAction::SetTrue)
                .help("Write an output header even if there is no input header"),
        )
        .arg(
            Arg::new("count-header")
                .long("count-header")
                .num_args(1)
                .help("Use STR as the header for count"),
        )
        .arg(
            Arg::new("values-delimiter")
                .long("values-delimiter")
                .short('v')
                .num_args(1)
                .default_value("|")
                .help("Delimiter for --unique and --collapse"),
        )
        .arg(
            Arg::new("float-precision")
                .long("float-precision")
                .short('p')
                .num_args(1)
                .default_value("4")
                .help("Precision for floating point numbers"),
        )
        .arg(
            Arg::new("exclude-missing")
                .long("exclude-missing")
                .short('x')
                .action(ArgAction::SetTrue)
                .help("Exclude missing (empty) fields from calculations"),
        )
        .arg(
            Arg::new("replace-missing")
                .long("replace-missing")
                .short('r')
                .num_args(1)
                .help("Replace missing values (nan) with a string"),
        )
        .arg(
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        );

    cmd
}

struct OpConfig {
    kind: OpKind,
    spec: Option<String>,
    arg_index: usize,
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    // Parameter validation
    let delimiter_str = matches
        .get_one::<String>("delimiter")
        .map(|s| s.as_str())
        .unwrap_or("\t");
    let values_delimiter_str = matches
        .get_one::<String>("values-delimiter")
        .map(|s| s.as_str())
        .unwrap_or("|");

    if delimiter_str == values_delimiter_str {
        return Err(anyhow::anyhow!(
            "values delimiter cannot be the same as field delimiter"
        ));
    }

    if matches.get_flag("exclude-missing")
        && matches.get_one::<String>("replace-missing").is_some()
    {
        return Err(anyhow::anyhow!(
            "argument '--exclude-missing' cannot be used with '--replace-missing <replace-missing>'"
        ));
    }

    let mut op_configs = Vec::new();

    macro_rules! parse_op {
        ($name:expr, $kind:expr) => {
            if let Some(indices) = matches.indices_of($name) {
                for (i, val) in indices.zip(matches.get_many::<String>($name).unwrap()) {
                    op_configs.push(OpConfig {
                        kind: $kind,
                        spec: Some(val.clone()),
                        arg_index: i,
                    });
                }
            }
        };
    }

    // Collect operations
    parse_op!("sum", OpKind::Sum);
    parse_op!("mean", OpKind::Mean);
    parse_op!("min", OpKind::Min);
    parse_op!("max", OpKind::Max);
    parse_op!("median", OpKind::Median);
    parse_op!("stdev", OpKind::Stdev);
    parse_op!("variance", OpKind::Variance);
    parse_op!("mad", OpKind::Mad);
    parse_op!("first", OpKind::First);
    parse_op!("last", OpKind::Last);
    parse_op!("nunique", OpKind::NUnique);
    parse_op!("mode", OpKind::Mode);
    parse_op!("mode-count", OpKind::ModeCount);
    parse_op!("missing-count", OpKind::MissingCount);
    parse_op!("not-missing-count", OpKind::NotMissingCount);
    parse_op!("geomean", OpKind::GeoMean);
    parse_op!("harmmean", OpKind::HarmMean);
    parse_op!("q1", OpKind::Q1);
    parse_op!("q3", OpKind::Q3);
    parse_op!("iqr", OpKind::IQR);
    parse_op!("cv", OpKind::CV);
    parse_op!("range", OpKind::Range);
    parse_op!("values", OpKind::Collapse);
    parse_op!("unique-values", OpKind::Unique);
    parse_op!("rand", OpKind::Rand);

    if let Some(indices) = matches.indices_of("quantile") {
        for (i, val) in indices.zip(matches.get_many::<String>("quantile").unwrap()) {
            // val format: "fields:probs[:header]" e.g. "1,2:0.5,0.9:MyQ"
            let parts: Vec<&str> = val.split(':').collect();
            if parts.len() < 2 {
                return Err(anyhow::anyhow!("Invalid quantile syntax: {}", val));
            }
            let fields_spec = parts[0];
            let probs_spec = parts[1];

            let constructed_spec = if parts.len() > 2 {
                format!("{}:{}", fields_spec, parts[2])
            } else {
                fields_spec.to_string()
            };

            for p_str in probs_spec.split(',') {
                let p = p_str.parse::<f64>().map_err(|e| {
                    anyhow::anyhow!("Invalid probability {}: {}", p_str, e)
                })?;
                if !(0.0..=1.0).contains(&p) {
                    return Err(anyhow::anyhow!(
                        "probability must be between 0.0 and 1.0: {}",
                        p
                    ));
                }
                op_configs.push(OpConfig {
                    kind: OpKind::Quantile(p),
                    spec: Some(constructed_spec.clone()),
                    arg_index: i,
                });
            }
        }
    }

    if matches.get_flag("count") {
        op_configs.push(OpConfig {
            kind: OpKind::Count,
            spec: None,
            arg_index: 0,
        });
    } else if matches.get_one::<String>("count-header").is_some() {
        op_configs.push(OpConfig {
            kind: OpKind::Count,
            spec: None,
            arg_index: 0,
        });
    }

    op_configs.sort_by_key(|c| c.arg_index);

    let mut config = StatsConfig::default();
    if let Some(p) = matches.get_one::<String>("replace-missing") {
        config.missing_val = Some(p.clone());
        if let Ok(v) = p.parse::<f64>() {
            config.missing_val_f64 = Some(v);
        }
    }
    if let Some(d) = matches.get_one::<String>("values-delimiter") {
        if let Some(c) = d.chars().next() {
            config.delimiter = c;
        }
    }
    if let Some(p) = matches.get_one::<String>("float-precision") {
        if let Ok(v) = p.parse::<usize>() {
            config.precision = Some(v);
        }
    }
    config.exclude_missing = matches.get_flag("exclude-missing");

    let infiles: Vec<String> = match matches.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let header_mode = matches.get_flag("header");
    let write_header = matches.get_flag("write-header");
    let group_by_spec = matches.get_one::<String>("group-by").cloned();
    let replace_missing = matches.get_one::<String>("replace-missing").cloned();
    let count_header = matches.get_one::<String>("count-header").cloned();

    let delimiter = if let Some(d) = matches.get_one::<String>("delimiter") {
        if let Some(c) = d.chars().next() {
            c as u8
        } else {
            b'\t'
        }
    } else {
        b'\t'
    };

    let mut processor: Option<StatsProcessor> = None;
    let mut aggregator: Option<Aggregator> = None;
    let mut groups: IndexMap<KeyBuffer, Aggregator> = IndexMap::new();
    let mut group_extractor: Option<KeyExtractor> = None;
    let mut use_grouping = false;

    // Helper to setup processor
    let setup_processor = |header_opt: Option<&fields::Header>| -> anyhow::Result<(
        StatsProcessor,
        Option<KeyExtractor>,
        Vec<String>,
    )> {
        let mut ops = Vec::new();
        let mut output_headers = Vec::new();

        for config in &op_configs {
            if let OpKind::Count = config.kind {
                ops.push(Operation {
                    kind: OpKind::Count,
                    field_idx: None,
                });
                output_headers
                    .push(count_header.clone().unwrap_or_else(|| "count".to_string()));
            } else if let Some(spec) = &config.spec {
                // Check if there is a custom header override (suffix ":header")
                let (field_spec, custom_header_override) =
                    if let Some(idx) = spec.rfind(':') {
                        // Check if the part after ':' is likely a probability (e.g. for quantile 1:0.5)
                        // Quantile configs are created earlier, and their spec is "fields_spec" (e.g. "1,2").
                        // So this block handles non-quantile ops where spec comes directly from arg.
                        // e.g. --sum 1:Header -> spec="1:Header".

                        let suffix = &spec[idx + 1..];
                        if suffix.is_empty() {
                            (spec.as_str(), None)
                        } else {
                            (&spec[..idx], Some(suffix.to_string()))
                        }
                    } else {
                        (spec.as_str(), None)
                    };

                let indices = fields::parse_field_list_with_header(
                    field_spec,
                    header_opt,
                    delimiter as char,
                )
                .map_err(|e| anyhow::anyhow!("Error parsing field list: {}", e))?;

                if custom_header_override.is_some() && indices.len() > 1 {
                    return Err(anyhow::anyhow!(
                        "custom header is not allowed with multiple fields"
                    ));
                }

                for idx in &indices {
                    let field_idx = *idx - 1;
                    ops.push(Operation {
                        kind: config.kind.clone(),
                        field_idx: Some(field_idx),
                    });

                    let suffix = match config.kind {
                        OpKind::Sum => "_sum".to_string(),
                        OpKind::Mean => "_mean".to_string(),
                        OpKind::Min => "_min".to_string(),
                        OpKind::Max => "_max".to_string(),
                        OpKind::Median => "_median".to_string(),
                        OpKind::Stdev => "_stdev".to_string(),
                        OpKind::Variance => "_variance".to_string(),
                        OpKind::Mad => "_mad".to_string(),
                        OpKind::First => "_first".to_string(),
                        OpKind::Last => "_last".to_string(),
                        OpKind::NUnique => "_nunique".to_string(),
                        OpKind::Mode => "_mode".to_string(),
                        OpKind::GeoMean => "_geomean".to_string(),
                        OpKind::HarmMean => "_harmmean".to_string(),
                        OpKind::Q1 => "_q1".to_string(),
                        OpKind::Q3 => "_q3".to_string(),
                        OpKind::IQR => "_iqr".to_string(),
                        OpKind::CV => "_cv".to_string(),
                        OpKind::Range => "_range".to_string(),
                        OpKind::Unique => "_unique".to_string(),
                        OpKind::Collapse => "_collapse".to_string(),
                        OpKind::Rand => "_rand".to_string(),
                        OpKind::ModeCount => "_mode_count".to_string(),
                        OpKind::MissingCount => "_missing_count".to_string(),
                        OpKind::NotMissingCount => "_not_missing_count".to_string(),
                        OpKind::Quantile(p) => format!("_quantile_{}", p),
                        OpKind::Count => "".to_string(),
                    };

                    let name = if let Some(custom) = &custom_header_override {
                        if indices.len() == 1 {
                            custom.clone()
                        } else {
                            format!("{}_{}", custom, idx)
                        }
                    } else if let Some(h) = header_opt {
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
            let idxs = fields::parse_field_list_with_header(
                spec,
                header_opt,
                delimiter as char,
            )
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

        Ok((
            StatsProcessor::new(ops, config.clone()),
            extractor,
            final_headers,
        ))
    };

    for input in crate::libs::io::raw_input_sources(&infiles) {
        let mut reader = TsvReader::with_capacity(input.reader, 512 * 1024);

        if header_mode {
            let header_bytes_opt = reader.read_header().map_err(map_io_err)?;
            if processor.is_none() {
                if let Some(header_bytes) = header_bytes_opt {
                    let line = String::from_utf8_lossy(&header_bytes);
                    let header = fields::Header::from_line(&line, delimiter as char);

                    let (proc, extractor, headers) = setup_processor(Some(&header))?;
                    processor = Some(proc);
                    group_extractor = extractor;
                    use_grouping = group_extractor.is_some();

                    if !use_grouping {
                        aggregator =
                            Some(processor.as_ref().unwrap().create_aggregator());
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
                        aggregator =
                            Some(processor.as_ref().unwrap().create_aggregator());
                    }

                    println!("{}", headers.join("\t"));
                }
            }
        } else {
            if processor.is_none() {
                let (proc, extractor, headers) = setup_processor(None)?;
                processor = Some(proc);
                group_extractor = extractor;
                use_grouping = group_extractor.is_some();

                if !use_grouping {
                    aggregator = Some(processor.as_ref().unwrap().create_aggregator());
                }

                if write_header {
                    println!("{}", headers.join("\t"));
                }
            }
        }

        if let Some(proc) = &processor {
            reader
                .for_each_row(delimiter, |row| {
                    if use_grouping {
                        let key_res = group_extractor
                            .as_mut()
                            .unwrap()
                            .extract_from_row(row, delimiter);
                        let key = match key_res {
                            Ok(k) => k.into_owned(),
                            Err(_) => KeyBuffer::new(),
                        };

                        let agg = groups
                            .entry(key)
                            .or_insert_with(|| proc.create_aggregator());
                        proc.update(agg, row);
                    } else {
                        if let Some(agg) = &mut aggregator {
                            proc.update(agg, row);
                        }
                    }
                    Ok(())
                })
                .map_err(map_io_err)?;
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
                let values = if let Some(replacement) = &replace_missing {
                    values
                        .into_iter()
                        .map(|v| if v == "nan" { replacement.clone() } else { v })
                        .collect()
                } else {
                    values
                };

                if !values.is_empty() {
                    print!("\t{}", values.join("\t"));
                }
                println!();
            }
        } else {
            if let Some(agg) = &aggregator {
                let values = proc.format_results(agg);
                let values = if let Some(replacement) = &replace_missing {
                    values
                        .into_iter()
                        .map(|v| if v == "nan" { replacement.clone() } else { v })
                        .collect()
                } else {
                    values
                };
                println!("{}", values.join("\t"));
            }
        }
    }

    Ok(())
}
