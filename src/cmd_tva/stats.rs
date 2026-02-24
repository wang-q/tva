use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::HashMap;
use std::io::BufRead;
use crate::libs::io::reader;
use crate::libs::fields;

pub fn make_subcommand() -> Command {
    Command::new("stats")
        .about("Calculates summary statistics (like tsv-summarize)")
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
            Arg::new("infiles")
                .num_args(0..)
                .index(1)
                .help("Input TSV file(s) to process (default: stdin)"),
        )
}

#[derive(Debug, Clone, PartialEq)]
enum OpKind {
    Count,
    Sum,
    Mean,
    Min,
    Max,
}

struct OpConfig {
    kind: OpKind,
    spec: Option<String>,
    arg_index: usize,
}

struct Operation {
    kind: OpKind,
    field_idx: Option<usize>, // None for count
}

struct Aggregator {
    count: usize,
    sums: HashMap<usize, f64>,
    mins: HashMap<usize, f64>,
    maxs: HashMap<usize, f64>,
    field_counts: HashMap<usize, usize>,
}

impl Aggregator {
    fn new() -> Self {
        Self {
            count: 0,
            sums: HashMap::new(),
            mins: HashMap::new(),
            maxs: HashMap::new(),
            field_counts: HashMap::new(),
        }
    }

    fn update(&mut self, record: &[&[u8]], ops: &[Operation]) {
        self.count += 1;

        // Handle Sum/Mean/FieldCounts separately to ensure single update per field per row
        // Collect fields needed for sum/mean
        let mut sum_fields = Vec::new();
        for op in ops {
            if matches!(op.kind, OpKind::Sum | OpKind::Mean) {
                if let Some(idx) = op.field_idx {
                    sum_fields.push(idx);
                }
            }
        }
        sum_fields.sort_unstable();
        sum_fields.dedup();

        for idx in sum_fields {
            if idx >= record.len() { continue; }
            let val_bytes = record[idx];
            if val_bytes.is_empty() { continue; }
            if let Ok(val_str) = std::str::from_utf8(val_bytes) {
                if let Ok(val) = val_str.trim().parse::<f64>() {
                    *self.sums.entry(idx).or_insert(0.0) += val;
                    *self.field_counts.entry(idx).or_insert(0) += 1;
                }
            }
        }

        // Handle Min/Max
        for op in ops {
            if let Some(idx) = op.field_idx {
                if idx >= record.len() { continue; }
                let val_bytes = record[idx];
                if val_bytes.is_empty() { continue; }

                if matches!(op.kind, OpKind::Min | OpKind::Max) {
                    if let Ok(val_str) = std::str::from_utf8(val_bytes) {
                        if let Ok(val) = val_str.trim().parse::<f64>() {
                            match op.kind {
                                OpKind::Min => {
                                    let entry = self.mins.entry(idx).or_insert(f64::INFINITY);
                                    if val < *entry {
                                        *entry = val;
                                    }
                                }
                                OpKind::Max => {
                                    let entry = self.maxs.entry(idx).or_insert(f64::NEG_INFINITY);
                                    if val > *entry {
                                        *entry = val;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    let mut op_configs = Vec::new();

    // Collect operations
    if let Some(indices) = matches.indices_of("sum") {
        for (i, val) in indices.zip(matches.get_many::<String>("sum").unwrap()) {
            op_configs.push(OpConfig { kind: OpKind::Sum, spec: Some(val.clone()), arg_index: i });
        }
    }
    if let Some(indices) = matches.indices_of("mean") {
        for (i, val) in indices.zip(matches.get_many::<String>("mean").unwrap()) {
            op_configs.push(OpConfig { kind: OpKind::Mean, spec: Some(val.clone()), arg_index: i });
        }
    }
    if let Some(indices) = matches.indices_of("min") {
        for (i, val) in indices.zip(matches.get_many::<String>("min").unwrap()) {
            op_configs.push(OpConfig { kind: OpKind::Min, spec: Some(val.clone()), arg_index: i });
        }
    }
    if let Some(indices) = matches.indices_of("max") {
        for (i, val) in indices.zip(matches.get_many::<String>("max").unwrap()) {
            op_configs.push(OpConfig { kind: OpKind::Max, spec: Some(val.clone()), arg_index: i });
        }
    }

    // Handle count.
    if matches.get_flag("count") {
        op_configs.push(OpConfig { kind: OpKind::Count, spec: None, arg_index: 0 });
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
                ops.push(Operation { kind: OpKind::Count, field_idx: None });
                output_headers.push("count".to_string());
            }
            _ => {
                if let Some(spec) = &config.spec {
                    let indices = fields::parse_field_list_with_header(spec, header.as_ref(), '\t')
                        .map_err(|e| anyhow::anyhow!("Error parsing field list: {}", e))?;

                    for idx in indices {
                        let field_idx = idx - 1;
                        ops.push(Operation { kind: config.kind.clone(), field_idx: Some(field_idx) });

                        let suffix = match config.kind {
                            OpKind::Sum => "_sum",
                            OpKind::Mean => "_mean",
                            OpKind::Min => "_min",
                            OpKind::Max => "_max",
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

            let mut line_buf = Vec::new();
            while file_reader.read_until(b'\n', &mut line_buf)? > 0 {
                if line_buf.ends_with(&[b'\n']) {
                    line_buf.pop();
                    if line_buf.ends_with(&[b'\r']) {
                        line_buf.pop();
                    }
                }

                let fields: Vec<&[u8]> = line_buf.split(|&c| c == b'\t').collect();

                let mut key = Vec::new();
                for (k_i, &idx) in group_indices.iter().enumerate() {
                    if k_i > 0 {
                        key.push(b'\t');
                    }
                    if idx < fields.len() {
                        key.extend_from_slice(fields[idx]);
                    }
                }

                let aggregator = groups.entry(key).or_insert_with(Aggregator::new);
                aggregator.update(&fields, &ops);

                line_buf.clear();
            }
        }

        let mut keys: Vec<_> = groups.keys().collect();
        keys.sort();

        for key in keys {
            let agg = &groups[key];
            print!("{}", String::from_utf8_lossy(key));

            let values = format_agg_results(agg, &ops);
            if !values.is_empty() {
                print!("\t{}", values.join("\t"));
            }
            println!();
        }

    } else {
        let mut aggregator = Aggregator::new();

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

            let mut line_buf = Vec::new();
            while file_reader.read_until(b'\n', &mut line_buf)? > 0 {
                if line_buf.ends_with(&[b'\n']) {
                    line_buf.pop();
                    if line_buf.ends_with(&[b'\r']) {
                        line_buf.pop();
                    }
                }

                let fields: Vec<&[u8]> = line_buf.split(|&c| c == b'\t').collect();
                aggregator.update(&fields, &ops);

                line_buf.clear();
            }
        }

        if !ops.is_empty() {
            let values = format_agg_results(&aggregator, &ops);
            println!("{}", values.join("\t"));
        }
    }

    Ok(())
}

fn format_agg_results(agg: &Aggregator, ops: &[Operation]) -> Vec<String> {
    let mut values = Vec::new();
    for op in ops {
        match op.kind {
            OpKind::Count => values.push(agg.count.to_string()),
            OpKind::Sum => {
                if let Some(idx) = op.field_idx {
                    let val = agg.sums.get(&idx).copied().unwrap_or(0.0);
                    values.push(val.to_string());
                }
            }
            OpKind::Mean => {
                if let Some(idx) = op.field_idx {
                    let sum = agg.sums.get(&idx).copied().unwrap_or(0.0);
                    let count = agg.field_counts.get(&idx).copied().unwrap_or(0);
                    if count > 0 {
                        values.push((sum / count as f64).to_string());
                    } else {
                        values.push("nan".to_string());
                    }
                }
            }
            OpKind::Min => {
                if let Some(idx) = op.field_idx {
                    let val = agg.mins.get(&idx).copied().unwrap_or(f64::INFINITY);
                    if val == f64::INFINITY {
                        values.push("".to_string());
                    } else {
                        values.push(val.to_string());
                    }
                }
            }
            OpKind::Max => {
                if let Some(idx) = op.field_idx {
                    let val = agg.maxs.get(&idx).copied().unwrap_or(f64::NEG_INFINITY);
                    if val == f64::NEG_INFINITY {
                        values.push("".to_string());
                    } else {
                        values.push(val.to_string());
                    }
                }
            }
        }
    }
    values
}
