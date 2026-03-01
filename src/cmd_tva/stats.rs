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
            Arg::new("var")
                .long("var")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate variance (alias for --variance)"),
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
            Arg::new("retain")
                .long("retain")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Retain one copy of the field (alias for --first)"),
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
                .visible_alias("unique-count")
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
            Arg::new("quantile")
                .long("quantile")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Calculate quantiles (e.g. field:0.25,0.75)"),
        )
        .arg(
            Arg::new("values")
                .long("values")
                .visible_alias("collapse")
                .num_args(1)
                .action(ArgAction::Append)
                .help("List all values of fields (comma separated)"),
        )
        .arg(
            Arg::new("unique-values")
                .long("unique-values")
                .visible_alias("unique")
                .num_args(1)
                .action(ArgAction::Append)
                .help("List unique values of fields (comma separated)"),
        )
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
            Arg::new("rand")
                .long("rand")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Pick a random value from fields"),
        )
        .arg(
            Arg::new("values-delimiter")
                .long("values-delimiter")
                .short('v')
                .num_args(1)
                .help("Delimiter for --unique and --collapse (default: ,)"),
        )
        .arg(
            Arg::new("float-precision")
                .long("float-precision")
                .short('p')
                .num_args(1)
                .help("Precision for floating point numbers (default: 4)"),
        )
        .arg(
            Arg::new("mode-count")
                .long("mode-count")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Count of the most frequent value"),
        )
        .arg(
            Arg::new("missing-count")
                .long("missing-count")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Number of missing (empty) fields"),
        )
        .arg(
            Arg::new("not-missing-count")
                .long("not-missing-count")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Number of filled (non-empty) fields"),
        )
        .arg(
            Arg::new("replace-missing")
                .long("replace-missing")
                .num_args(1)
                .help("Replace missing values (nan) with a string"),
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
    if let Some(indices) = matches.indices_of("var") {
        for (i, val) in indices.zip(matches.get_many::<String>("var").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Variance,
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
    if let Some(indices) = matches.indices_of("retain") {
        for (i, val) in indices.zip(matches.get_many::<String>("retain").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::First,
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
    if let Some(indices) = matches.indices_of("mode-count") {
        for (i, val) in indices.zip(matches.get_many::<String>("mode-count").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::ModeCount,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("missing-count") {
        for (i, val) in indices.zip(matches.get_many::<String>("missing-count").unwrap())
        {
            op_configs.push(OpConfig {
                kind: OpKind::MissingCount,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }
    if let Some(indices) = matches.indices_of("not-missing-count") {
        for (i, val) in
            indices.zip(matches.get_many::<String>("not-missing-count").unwrap())
        {
            op_configs.push(OpConfig {
                kind: OpKind::NotMissingCount,
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
                if p < 0.0 || p > 1.0 {
                    return Err(anyhow::anyhow!(
                        "Probability must be between 0 and 1: {}",
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

    if let Some(indices) = matches.indices_of("values") {
        for (i, val) in indices.zip(matches.get_many::<String>("values").unwrap()) {
            op_configs.push(OpConfig {
                kind: OpKind::Collapse,
                spec: Some(val.clone()),
                arg_index: i,
            });
        }
    }

    if let Some(indices) = matches.indices_of("unique-values") {
        for (i, val) in indices.zip(matches.get_many::<String>("unique-values").unwrap())
        {
            op_configs.push(OpConfig {
                kind: OpKind::Unique,
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
    if matches.get_flag("count") || matches.get_one::<String>("count-header").is_some() {
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

    let infiles: Vec<String> = match matches.get_many::<String>("infiles") {
        Some(values) => values.cloned().collect(),
        None => vec!["stdin".to_string()],
    };

    let header_mode = matches.get_flag("header");
    let write_header = matches.get_flag("write-header");
    let group_by_spec = matches.get_one::<String>("group-by").cloned();
    let replace_missing = matches.get_one::<String>("replace-missing").cloned();
    let count_header = matches.get_one::<String>("count-header").cloned();

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
                let (field_spec, custom_header_override) = if let Some(idx) = spec.rfind(':') {
                    // Check if the part after ':' is likely a probability (e.g. for quantile 1:0.5)
                    // Quantile configs are created earlier, and their spec is "fields_spec" (e.g. "1,2").
                    // But if the user typed --quantile 1:0.5:Header, the first split (in argument parsing loop)
                    // would have split at the first ':', giving fields="1" and probs="0.5:Header".
                    // Then probs parsing would fail.
                    // So for quantile, we might need to handle headers differently or disallow them in the same arg.
                    // tsv-summarize supports --quantile 1:0.5:Header.
                    // My current quantile parsing loop splits by ':' (limit 2? no).
                    // Let's re-examine quantile parsing loop.
                    
                    // But here, 'spec' is the field list string.
                    // For quantile, I set spec = fields_spec.
                    // If fields_spec contained a header, it would have been split out earlier?
                    // No, quantile parsing loop:
                    // let parts: Vec<&str> = val.split(':').collect();
                    // fields_spec = parts[0]; probs_spec = parts[1];
                    // If val="1:0.5:Header", parts=["1", "0.5", "Header"].
                    // My current logic takes parts[0] and parts[1]. Ignores parts[2]?
                    // No, I check parts.len() < 2. I don't check > 2.
                    // So for quantile, spec is just parts[0] ("1").
                    // So this block handles non-quantile ops where spec comes directly from arg.
                    // e.g. --sum 1:Header -> spec="1:Header".
                    
                    let suffix = &spec[idx+1..];
                    if suffix.is_empty() {
                         (spec.as_str(), None)
                    } else {
                         (&spec[..idx], Some(suffix.to_string()))
                    }
                } else {
                    (spec.as_str(), None)
                };

                let indices =
                    fields::parse_field_list_with_header(field_spec, header_opt, '\t')
                        .map_err(|e| {
                            anyhow::anyhow!("Error parsing field list: {}", e)
                        })?;

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
                    let header = fields::Header::from_line(&line, '\t');

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
                .for_each_row(|row| {
                    if use_grouping {
                        let key_res = group_extractor
                            .as_mut()
                            .unwrap()
                            .extract_from_row(row, b'\t');
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
